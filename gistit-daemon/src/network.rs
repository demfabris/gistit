//! The network module
#![allow(clippy::missing_errors_doc)]

use std::collections::HashSet;
use std::str::FromStr;
use std::string::ToString;

use either::Either;
use gistit_ipc::{self, Bridge, Instruction, Server, ServerResponse};
use log::{debug, error, info, warn};

use libp2p::core::either::EitherError;
use libp2p::core::PeerId;
use libp2p::futures::StreamExt;
use libp2p::multiaddr::{multiaddr, Protocol};
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmBuilder, SwarmEvent};
use libp2p::{tokio_development_transport, Multiaddr, Swarm};

use libp2p::identify::{IdentifyEvent, IdentifyInfo};
use libp2p::kad::{self, GetProvidersOk, KademliaEvent, QueryId, QueryResult};
use libp2p::ping::Failure;

use crate::behaviour::{Behaviour, Event};
use crate::config::Config;
use crate::Result;

/// The main event loop
pub struct Node {
    swarm: Swarm<Behaviour>,
    bridge: Bridge<Server>,
    pending_dial: HashSet<PeerId>,
    pending_start_providing: HashSet<QueryId>,
    pending_get_providers: HashSet<QueryId>,
    // pending_request_file: HashMap<RequestId, oneshot::Sender<Result<String>>>,
}

const BOOTADDR: &str = "/dnsaddr/bootstrap.libp2p.io";
pub const GISTIT_RELAY_NODE: &str = "12D3KooWJtJX9qwBdECWoLk2hktSLFBcHSUWpP4FiN3u4Hns7347";
pub const GISTIT_BOOTADDR: &str = "/ip4/34.125.73.67/tcp/4001";

impl Node {
    pub async fn new(config: Config) -> Result<Self> {
        let behaviour = Behaviour::new(&config)?;
        let transport = tokio_development_transport(config.keypair)?;

        let mut swarm = SwarmBuilder::new(transport, behaviour, config.peer_id)
            .executor(Box::new(|fut| {
                tokio::task::spawn(fut);
            }))
            .build();
        let bridge = gistit_ipc::server(&config.runtime_dir)?;

        let relay = GISTIT_BOOTADDR
            .parse::<Multiaddr>()
            .expect("valid multiaddr")
            .with(Protocol::P2p(
                PeerId::from_str(GISTIT_RELAY_NODE)
                    .expect("valid peerid string")
                    .into(),
            ));
        swarm.dial(relay)?;

        Ok(Self {
            swarm,
            bridge,
            pending_dial: HashSet::default(),
            pending_start_providing: HashSet::default(),
            pending_get_providers: HashSet::default(),
        })
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                swarm_event = self.swarm.next() => self.handle_swarm_event(
                    swarm_event.expect("swarm stream not to end")).await?,

                bridge_event = self.bridge.recv() => self.handle_bridge_event(bridge_event?).await?
            }
        }
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
                            EitherError<ProtocolsHandlerUpgrErr<std::io::Error>, std::io::Error>,
                            std::io::Error,
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
                    ProtocolsHandlerUpgrErr<std::io::Error>,
                >,
                Failure,
            >,
        >,
    ) -> Result<()> {
        match event {
            SwarmEvent::Behaviour(Event::Identify(IdentifyEvent::Received {
                peer_id,
                info:
                    IdentifyInfo {
                        listen_addrs,
                        protocols,
                        ..
                    },
            })) => {
                debug!("Identify: {:?}", listen_addrs);
                if protocols
                    .iter()
                    .any(|p| p.as_bytes() == kad::protocol::DEFAULT_PROTO_NAME)
                {
                    for addr in listen_addrs {
                        self.swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, addr);
                    }
                }
            }

            // Kademlia events
            SwarmEvent::Behaviour(Event::Kademlia(KademliaEvent::OutboundQueryCompleted {
                id,
                result: QueryResult::StartProviding(_),
                ..
            })) => {
                info!("Start providing in kad");
                self.pending_start_providing.remove(&id);
            }
            SwarmEvent::Behaviour(Event::Kademlia(KademliaEvent::OutboundQueryCompleted {
                id,
                result: QueryResult::GetProviders(Ok(GetProvidersOk { providers, .. })),
                ..
            })) => {
                info!("Got providers: {:?}", providers);
                self.pending_get_providers.remove(&id);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Daemon: Listening on {:?}", address);

                let peer_id = self.swarm.local_peer_id().to_string();

                self.bridge.connect_blocking()?;
                self.bridge
                    .send(Instruction::Response(ServerResponse::PeerId(peer_id)))
                    .await?;
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
            ev => {
                info!("other event: {:?}", ev);
            }
        }
        Ok(())
    }

    async fn handle_bridge_event(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::Listen { host, port } => {
                info!("Instruction: Listen");
                let address = multiaddr!(Ip4(host), Tcp(port));
                self.swarm.listen_on(address)?;
            }

            Instruction::Dial { peer_id } => {
                info!("Instruction: Dial");

                let base_addr: Multiaddr = BOOTADDR.parse().unwrap();
                let relay_addr: Multiaddr = "/ip4/34.125.73.67/tcp/4001/p2p/12D3KooWJtJX9qwBdECWoLk2hktSLFBcHSUWpP4FiN3u4Hns7347/p2p-circuit".parse().unwrap();
                let peer: PeerId = peer_id.parse().unwrap();

                if self.pending_dial.contains(&peer) {
                    error!("Already dialing peer: {}", peer_id);
                    return Ok(());
                }

                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer, base_addr);

                self.swarm
                    .dial(relay_addr.with(Protocol::P2p(peer.into())))?;
                self.pending_dial.insert(peer);
            }

            Instruction::Provide { hash, .. } => {
                info!("Instruction: Provide file {}", hash);

                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(hash.into_bytes().into())
                    .expect("to start providing");
            }

            Instruction::Get { hash } => {
                info!("Instruction: Get providers for {}", hash);

                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(hash.into_bytes().into());
                self.pending_get_providers.insert(query_id);
            }

            Instruction::Status => {
                info!("Instruction: Status");

                let listeners: Vec<String> =
                    self.swarm.listeners().map(ToString::to_string).collect();
                let network_info = self.swarm.network_info();

                self.bridge.connect_blocking()?;
                self.bridge
                    .send(Instruction::Response(ServerResponse::Status(format!(
                        "listeners: {:?}, network: {:?}",
                        listeners, network_info
                    ))))
                    .await?;
            }

            Instruction::Shutdown => {
                warn!("Exiting...");
                std::process::exit(0);
            }
            _ => (),
        }
        Ok(())
    }
}
