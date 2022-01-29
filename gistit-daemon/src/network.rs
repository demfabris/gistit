//! The network module
#![allow(clippy::missing_errors_doc)]

use std::collections::HashSet;
use std::iter::once;
use std::str::FromStr;
use std::string::ToString;
use std::time::Duration;

use either::Either;
use gistit_ipc::{self, Bridge, Instruction, Server, ServerResponse};
use log::{debug, info, trace, warn};
use void::Void;

use libp2p::core::either::EitherError;
use libp2p::core::upgrade;
use libp2p::core::PeerId;
use libp2p::futures::StreamExt;
use libp2p::multiaddr::{multiaddr, Protocol};
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmEvent};
use libp2p::{autonat, dns, mplex, noise, tcp, websocket, yamux, Multiaddr, Swarm, Transport};

use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent, IdentifyInfo};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{
    self, GetProvidersOk, Kademlia, KademliaConfig, KademliaEvent, QueryId, QueryResult,
};
use libp2p::relay::v2::relay;
use libp2p::request_response::{ProtocolSupport, RequestResponse, RequestResponseConfig};

use crate::behaviour::{Behaviour, Event, ExchangeCodec, ExchangeProtocol};
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

const BOOTNODES: [&str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

const BOOTADDR: &str = "/dnsaddr/bootstrap.libp2p.io";

impl Node {
    pub async fn new(config: Config) -> Result<Self> {
        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&config.keypair)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = {
            let tcp = tcp::TcpConfig::new().nodelay(true);
            let dns_tcp = dns::DnsConfig::system(tcp).await?;
            let ws_dns_tcp = websocket::WsConfig::new(dns_tcp.clone());
            dns_tcp
                .or_transport(ws_dns_tcp)
                .upgrade(upgrade::Version::V1)
                .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
                .multiplex(upgrade::SelectUpgrade::new(
                    yamux::YamuxConfig::default(),
                    mplex::MplexConfig::default(),
                ))
                .timeout(Duration::from_secs(20))
                .boxed()
        };

        let request_response = RequestResponse::new(
            ExchangeCodec,
            once((ExchangeProtocol, ProtocolSupport::Full)),
            RequestResponseConfig::default(),
        );

        let kademlia = {
            let mut cfg = KademliaConfig::default();
            cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let store = MemoryStore::new(config.peer_id);
            let mut behaviour = Kademlia::with_config(config.peer_id, store, cfg);

            let bootaddr = Multiaddr::from_str(BOOTADDR)?;
            for peer in &BOOTNODES {
                behaviour.add_address(
                    &PeerId::from_str(peer).expect("peer id to be valid"),
                    bootaddr.clone(),
                );
            }

            behaviour.bootstrap().expect("to bootstrap");
            behaviour
        };

        let identify = Identify::new(IdentifyConfig::new(
            "/ipfs/0.1.0".into(),
            config.keypair.public(),
        ));

        let relay = relay::Relay::new(
            PeerId::from(config.keypair.public()),
            relay::Config::default(),
        );
        let autonat = autonat::Behaviour::new(
            PeerId::from(config.keypair.public()),
            autonat::Config::default(),
        );

        let swarm = Swarm::new(
            transport,
            Behaviour {
                request_response,
                kademlia,
                identify,
                relay,
                autonat,
            },
            config.peer_id,
        );

        let bridge = gistit_ipc::server(&config.runtime_dir)?;

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
                    swarm_event.expect("to recv swarm event")).await?,

                bridge_event = async { self.bridge.recv() } => self.handle_bridge_event(bridge_event?).await?
            }
        }
    }

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<
            Event,
            EitherError<
                EitherError<
                    EitherError<
                        EitherError<ProtocolsHandlerUpgrErr<std::io::Error>, std::io::Error>,
                        std::io::Error,
                    >,
                    Either<
                        ProtocolsHandlerUpgrErr<
                            EitherError<impl std::error::Error, impl std::error::Error>,
                        >,
                        Void,
                    >,
                >,
                ProtocolsHandlerUpgrErr<std::io::Error>,
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
            //
            // Kademlia events
            //
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
                    .send(Instruction::Response(ServerResponse::PeerId(peer_id)))?;
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                debug!("Connection established {:?}", peer_id);
                if endpoint.is_dialer() {
                    self.pending_dial.remove(&peer_id);
                }
            }
            SwarmEvent::OutgoingConnectionError {
                peer_id: maybe_peer_id,
                error,
                ..
            } => {
                debug!("Outgoing connection error: {:?}", error);
                if let Some(peer_id) = maybe_peer_id {
                    self.pending_dial.remove(&peer_id);
                }
            }
            ev => {
                trace!("other event: {:?}", ev);
            }
        }
        Ok(())
    }

    async fn handle_bridge_event(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::Listen { host, port } => {
                info!("Instruction: Listen");
                let addr = multiaddr!(Ip4(host), Tcp(port));
                self.swarm.listen_on(addr)?;
            }
            Instruction::Dial { peer_id } => {
                info!("Instruction: Dial");

                // let addr: Multiaddr = BOOTADDR.parse().unwrap();
                let addr: Multiaddr = "/ip4/192.168.1.77/tcp/4001".parse().unwrap();
                let peer: PeerId = peer_id.parse().unwrap();

                if self.pending_dial.contains(&peer) {
                    debug!("Already dialing peer: {}", peer_id);
                    return Ok(());
                }

                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer, addr.clone());

                self.swarm.dial(addr.with(Protocol::P2p(peer.into())))?;
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
                    ))))?;
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
