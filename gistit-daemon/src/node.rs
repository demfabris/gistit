//! The network module
#![allow(clippy::missing_errors_doc)]

use std::collections::{HashMap, HashSet};
use std::io;
use std::string::ToString;
use std::task::Poll;

use either::Either;
use log::{debug, error, info, warn};

use gistit_ipc::{Bridge, Server};
use gistit_proto::{ipc, Gistit, Instruction};

use libp2p::core::either::EitherError;
use libp2p::core::{self, Multiaddr, PeerId};
use libp2p::futures::future::poll_fn;
use libp2p::futures::StreamExt;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmBuilder, SwarmEvent};
use libp2p::{dns, mplex, noise, tcp, websocket, yamux, Swarm, Transport};

use libp2p::kad::{record::Key, QueryId};
use libp2p::ping::Failure;
use libp2p::request_response::RequestId;

use crate::behaviour::{Behaviour, Event, Request};
use crate::config::Config;
use crate::event::{handle_identify, handle_kademlia, handle_request_response};
use crate::Result;

/// The main event loop
pub struct Node {
    pub swarm: Swarm<Behaviour>,
    pub bridge: Bridge<Server>,

    pub pending_dial: HashSet<PeerId>,

    /// Pending kademlia queries to get providers
    pub pending_get_providers: HashSet<QueryId>,

    pub pending_start_providing: HashSet<QueryId>,
    pub to_provide: HashMap<Key, Gistit>,

    pub pending_request_file: HashSet<RequestId>,

    /// Stack of request file (`key`) events
    pub to_request: Vec<(Key, HashSet<PeerId>)>,
    pub pending_receive_file: HashSet<Key>,

    /// Addresses that can be used as relay
    pub relays: HashSet<Multiaddr>,
}

impl Node {
    pub async fn new(config: Config) -> Result<Self> {
        let (behaviour, client_transport) = Behaviour::new_behaviour_and_transport(&config)?;

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&config.keypair)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = {
            let tcp = tcp::TokioTcpConfig::new().nodelay(true);
            let dns_tcp = dns::TokioDnsConfig::system(tcp.clone())?;
            let ws_dns_tcp = websocket::WsConfig::new(tcp.clone());

            tcp.or_transport(client_transport)
                .or_transport(dns_tcp)
                .or_transport(ws_dns_tcp)
                .upgrade(core::upgrade::Version::V1)
                .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
                .multiplex(core::upgrade::SelectUpgrade::new(
                    yamux::YamuxConfig::default(),
                    mplex::MplexConfig::default(),
                ))
                .timeout(std::time::Duration::from_secs(20))
                .boxed()
        };

        let mut swarm = SwarmBuilder::new(transport, behaviour, config.peer_id)
            .executor(Box::new(|fut| {
                tokio::task::spawn(fut);
            }))
            .build();
        swarm.listen_on(config.multiaddr)?;

        let bridge = gistit_ipc::server(&config.runtime_path)?;

        Ok(Self {
            swarm,
            bridge,
            pending_dial: HashSet::default(),
            pending_start_providing: HashSet::default(),
            pending_get_providers: HashSet::default(),
            pending_request_file: HashSet::default(),
            pending_receive_file: HashSet::default(),

            to_provide: HashMap::default(),
            to_request: Vec::default(),

            relays: HashSet::default(),
        })
    }

    pub fn dial_on_init(&mut self, address: &str) -> Result<()> {
        Ok(self.swarm.dial(address.parse::<Multiaddr>()?)?)
    }

    pub fn listen_on_init(&mut self, address: &str) -> Result<()> {
        self.swarm.listen_on(address.parse::<Multiaddr>()?)?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                swarm_event = self.swarm.next() => self.handle_swarm_event(
                    swarm_event.expect("stream not to end")).await?,

                bridge_event = self.bridge.recv() => self.handle_bridge_event(bridge_event?).await?,

                request_event = poll_fn(|_| {
                    self.to_request.pop().map_or(Poll::Pending, Poll::Ready)
                }) => self.handle_request_event(request_event).await?,
            }
        }
    }

    async fn handle_request_event(&mut self, event: (Key, HashSet<PeerId>)) -> Result<()> {
        let (key, providers) = event;

        self.pending_receive_file.insert(key.clone());
        for peer in providers {
            for relay in &self.relays {
                // Skip if we are trying to relay over the destination peer itself
                if relay
                    .iter()
                    .any(|protocol| protocol == Protocol::P2p(peer.into()))
                {
                    continue;
                }

                self.swarm
                    .behaviour_mut()
                    .request_response
                    .add_address(&peer, relay.clone());
                // let relayed_addr = relay.clone().with(Protocol::P2p(peer.into()));
                // self.swarm.dial(relayed_addr)?;
            }

            let request_id = self
                .swarm
                .behaviour_mut()
                .request_response
                .send_request(&peer, Request(key.to_vec()));
            info!("Requesting gistit from {:?}", peer);

            self.pending_request_file.insert(request_id);
        }

        Ok(())
    }

    #[allow(clippy::type_complexity)]
    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<
            Event,
            EitherError<
                EitherError<
                    EitherError<
                        EitherError<
                            EitherError<
                                EitherError<ProtocolsHandlerUpgrErr<io::Error>, io::Error>,
                                io::Error,
                            >,
                            Either<
                                ProtocolsHandlerUpgrErr<
                                    EitherError<
                                        impl std::error::Error + Send,
                                        impl std::error::Error + Send,
                                    >,
                                >,
                                void::Void,
                            >,
                        >,
                        ProtocolsHandlerUpgrErr<io::Error>,
                    >,
                    Failure,
                >,
                Either<
                    ProtocolsHandlerUpgrErr<
                        EitherError<impl std::error::Error + Send, impl std::error::Error + Send>,
                    >,
                    void::Void,
                >,
            >,
        >,
    ) -> Result<()> {
        match event {
            SwarmEvent::Behaviour(Event::Identify(event)) => handle_identify(self, event)?,
            SwarmEvent::Behaviour(Event::Kademlia(event)) => handle_kademlia(self, event).await?,
            SwarmEvent::Behaviour(Event::RequestResponse(event)) => {
                handle_request_response(self, event).await?;
            }

            SwarmEvent::NewListenAddr { address, .. } => {
                let peer_id = self.swarm.local_peer_id().to_string();
                info!("Listening on {:?}, {:?}", address, peer_id);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                info!("Connection established {:?}", peer_id);
                if endpoint.is_dialer() {
                    self.pending_dial.remove(&peer_id);
                }
            }
            SwarmEvent::OutgoingConnectionError {
                peer_id: maybe_peer_id,
                error,
                ..
            } => {
                error!("Outgoing connection error: {:?}", error);
                if let Some(peer_id) = maybe_peer_id {
                    self.pending_dial.remove(&peer_id);
                }
            }
            SwarmEvent::Behaviour(Event::Relay(e)) => warn!("{:?}", e),
            SwarmEvent::Behaviour(Event::Ping(_)) => {}
            // SwarmEvent::Behaviour(Event::Autonat(e)) => warn!("{:?}", e),
            ev => {
                debug!("other event: {:?}", ev);
            }
        }
        Ok(())
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    #[allow(clippy::cast_possible_truncation)]
    async fn handle_bridge_event(&mut self, instruction: Instruction) -> Result<()> {
        match instruction.expect_request()? {
            ipc::instruction::Kind::ProvideRequest(ipc::instruction::ProvideRequest {
                gistit: Some(gistit),
            }) => {
                warn!("Instruction: Provide gistit {}", &gistit.hash);
                let key = Key::new(&gistit.hash);

                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(key.clone())
                    .expect("to start providing");

                self.pending_start_providing.insert(query_id);
                self.to_provide.insert(key, gistit);
            }

            ipc::instruction::Kind::FetchRequest(ipc::instruction::FetchRequest { hash }) => {
                warn!("Instruction: Get providers for {}", hash);
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(Key::new(&hash));
                self.pending_get_providers.insert(query_id);
            }

            ipc::instruction::Kind::StatusRequest(ipc::instruction::StatusRequest {}) => {
                warn!("Instruction: Status");

                let network_info = self.swarm.network_info();

                let peer_id = self.swarm.local_peer_id().to_string();
                let peer_count = network_info.num_peers() as u32;
                let pending_connections = network_info.connection_counters().num_pending();
                let hosting = self.to_provide.len() as u32;

                self.bridge.connect_blocking()?;
                self.bridge
                    .send(Instruction::respond_status(
                        peer_id,
                        peer_count,
                        pending_connections,
                        hosting,
                    ))
                    .await?;
            }

            ipc::instruction::Kind::DialRequest(ipc::instruction::DialRequest { address }) => {
                warn!("Instruction: Dial");
                let multiaddr: Multiaddr = address.parse()?;
                self.swarm.dial(multiaddr)?;
            }

            ipc::instruction::Kind::ShutdownRequest(ipc::instruction::ShutdownRequest {}) => {
                warn!("Exiting...");
                std::process::exit(0);
            }

            _ => (),
        }
        Ok(())
    }
}
