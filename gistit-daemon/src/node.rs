//! The network module
#![allow(clippy::missing_errors_doc)]

use std::collections::{HashMap, HashSet};
use std::string::ToString;
use std::task::Poll;

use either::Either;
use gistit_ipc::{self, Bridge, Instruction, Server, ServerResponse};
use gistit_reference::Gistit;
use log::{debug, error, info, warn};

use libp2p::core::either::EitherError;
use libp2p::core::{Multiaddr, PeerId};
use libp2p::futures::future::poll_fn;
use libp2p::futures::StreamExt;
use libp2p::multiaddr::multiaddr;
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmBuilder, SwarmEvent};
use libp2p::{tokio_development_transport, Swarm};

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
}

impl Node {
    pub async fn new(config: Config) -> Result<Self> {
        let behaviour = Behaviour::new(&config)?;
        let transport = tokio_development_transport(config.keypair)?;

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

            to_provide: HashMap::default(),
            to_request: Vec::default(),
        })
    }

    pub async fn run(mut self) -> Result<()> {
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

        for p in providers {
            self.swarm.dial(p)?;

            let request_id = self
                .swarm
                .behaviour_mut()
                .request_response
                .send_request(&p, Request(key.to_vec()));
            info!("Requesting gistit from {:?}", p);

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
            SwarmEvent::Behaviour(Event::Identify(event)) => handle_identify(self, event),
            SwarmEvent::Behaviour(Event::Kademlia(event)) => handle_kademlia(self, event).await?,
            SwarmEvent::Behaviour(Event::RequestResponse(event)) => {
                handle_request_response(self, event).await?;
            }

            SwarmEvent::NewListenAddr { address, .. } => {
                let peer_id = self.swarm.local_peer_id().to_string();
                info!("Daemon: Listening on {:?}, {:?}", address, peer_id);
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
                debug!("other event: {:?}", ev);
            }
        }
        Ok(())
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    async fn handle_bridge_event(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::Provide { hash, data } => {
                warn!("Instruction: Provide gistit {}", hash);
                let key = Key::new(&hash);

                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(key.clone())
                    .expect("to start providing");

                //TODO: Clone file to stash directory

                self.pending_start_providing.insert(query_id);
                self.to_provide.insert(key, data);
            }

            Instruction::Fetch { hash } => {
                warn!("Instruction: Get providers for {}", hash);
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(Key::new(&hash));
                self.pending_get_providers.insert(query_id);
            }

            Instruction::Status => {
                warn!("Instruction: Status");

                let peer_id = self.swarm.local_peer_id().to_string();
                let listeners: Vec<String> =
                    self.swarm.listeners().map(ToString::to_string).collect();
                let network_info = self.swarm.network_info();
                let hosting = self.to_provide.len();

                self.bridge.connect_blocking()?;
                self.bridge
                    .send(Instruction::Response(ServerResponse::Status {
                        peer_count: network_info.num_peers(),
                        pending_connections: network_info.connection_counters().num_pending(),
                        peer_id,
                        listeners,
                        hosting,
                    }))
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
