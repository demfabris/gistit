//! The network module
#![allow(clippy::missing_errors_doc)]

use std::collections::{HashMap, HashSet};
use std::iter::once;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use either::Either;
use lib_gistit::ipc::{self, Bridge, Instruction, Server, ServerResponse};
use log::{debug, info, warn};
use void::Void;

use libp2p::core::either::EitherError;
use libp2p::core::upgrade::{self, read_length_prefixed, write_length_prefixed};
use libp2p::core::{PeerId, ProtocolName};
use libp2p::futures::{AsyncRead, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent, IdentifyInfo};
use libp2p::identity::Keypair;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{
    self, GetProvidersOk, Kademlia, KademliaConfig, KademliaEvent, QueryId, QueryResult,
};
use libp2p::multiaddr::{multiaddr, Protocol};
use libp2p::ping::{Ping, PingEvent, PingFailure};
use libp2p::relay::v2::relay;
use libp2p::request_response::{
    ProtocolSupport, RequestResponse, RequestResponseCodec, RequestResponseConfig,
    RequestResponseEvent,
};
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmEvent};
use libp2p::{
    autonat, dns, identity, mplex, noise, ping, tcp, websocket, yamux, Multiaddr, NetworkBehaviour,
    Swarm, Transport,
};

use crate::Result;

pub struct NetworkConfig {
    peer_id: PeerId,
    keypair: Keypair,
    runtime_dir: PathBuf,
}

impl NetworkConfig {
    pub fn new(seed: &str, runtime_dir: PathBuf) -> Result<Self> {
        let mut bytes: Vec<u8> = seed.as_bytes().to_vec();
        bytes.resize_with(32, || 0);
        let mut bytes: [u8; 32] = bytes.try_into().unwrap();

        let ed25519_secret = identity::ed25519::SecretKey::from_bytes(&mut bytes).unwrap();
        let keypair = identity::Keypair::Ed25519(ed25519_secret.into());

        let peer_id = PeerId::from(keypair.public());

        Ok(Self {
            peer_id,
            keypair,
            runtime_dir,
        })
    }

    pub async fn apply(self) -> Result<NetworkNode> {
        NetworkNode::new(self).await
    }
}

/// The main event loop
pub struct NetworkNode {
    swarm: Swarm<GistitNetworkBehaviour>,
    bridge: Bridge<Server>,
    pending_dial: HashSet<PeerId>,
    pending_start_providing: HashSet<QueryId>,
    // pending_get_providers: HashMap<QueryId, oneshot::Sender<HashSet<PeerId>>>,
    // pending_request_file: HashMap<RequestId, oneshot::Sender<Result<String>>>,
}

const BOOTNODES: [&'static str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

const BOOTADDR: &str = "/dnsaddr/bootstrap.libp2p.io";

impl NetworkNode {
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        let req_res = RequestResponse::new(
            GistitExchangeCodec,
            once((GistitExchangeProtocol, ProtocolSupport::Full)),
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

        let raw_transport = {
            let tcp = tcp::TcpConfig::new().nodelay(true);
            let dns_tcp = dns::DnsConfig::system(tcp).await?;
            let ws_dns_tcp = websocket::WsConfig::new(dns_tcp.clone());
            dns_tcp.or_transport(ws_dns_tcp)
        };

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&config.keypair)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = raw_transport
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(upgrade::SelectUpgrade::new(
                yamux::YamuxConfig::default(),
                mplex::MplexConfig::default(),
            ))
            .timeout(Duration::from_secs(60 * 30))
            .boxed();

        let identify = Identify::new(IdentifyConfig::new(
            "/ipfs/0.1.0".into(),
            config.keypair.public(),
        ));

        let ping = ping::Behaviour::new(ping::Config::new().with_keep_alive(true));
        let relay = relay::Relay::new(PeerId::from(config.keypair.public()), Default::default());
        let autonat =
            autonat::Behaviour::new(PeerId::from(config.keypair.public()), Default::default());

        let swarm = Swarm::new(
            transport,
            GistitNetworkBehaviour {
                req_res,
                kademlia,
                identify,
                ping,
                relay,
                autonat,
            },
            config.peer_id,
        );

        let bridge = ipc::server(&config.runtime_dir)?;

        Ok(Self {
            swarm,
            bridge,
            pending_dial: Default::default(),
            pending_start_providing: Default::default(),
        })
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                swarm_event = self.swarm.next() => self.handle_swarm_event(
                    swarm_event.expect("to recv swarm event")).await?,

                bridge_event = self.bridge.recv() => self.handle_bridge_event(bridge_event?).await?
            }
        }
    }

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<
            GistitNetworkEvent,
            EitherError<
                EitherError<
                    EitherError<
                        EitherError<
                            EitherError<ProtocolsHandlerUpgrErr<std::io::Error>, std::io::Error>,
                            std::io::Error,
                        >,
                        PingFailure,
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
            SwarmEvent::Behaviour(GistitNetworkEvent::Identify(IdentifyEvent::Received {
                peer_id,
                info:
                    IdentifyInfo {
                        listen_addrs,
                        protocols,
                        ..
                    },
            })) => {
                info!("Identify: {:?}", listen_addrs);
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
            SwarmEvent::Behaviour(GistitNetworkEvent::Kademlia(
                KademliaEvent::OutboundQueryCompleted {
                    id,
                    result: QueryResult::StartProviding(_),
                    ..
                },
            )) => {
                self.pending_start_providing.remove(&id);
            }
            SwarmEvent::Behaviour(GistitNetworkEvent::Kademlia(
                KademliaEvent::OutboundQueryCompleted {
                    id,
                    result: QueryResult::GetProviders(Ok(GetProvidersOk { providers, .. })),
                    ..
                },
            )) => {
                info!("Got providers: {:?}", providers);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                debug!("Daemon: Listening on {:?}", address);

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
                info!("Outgoing connection error: {:?}", error);
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
                debug!("Instruction: Listen");
                let addr = multiaddr!(Ip4(host), Tcp(port));
                self.swarm.listen_on(addr)?;
            }
            Instruction::Dial { peer_id } => {
                debug!("Instruction: Dial");

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
                debug!("Instruction: Provide file {}", hash);

                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(hash.into_bytes().into())
                    .expect("to start providing");
            }
            Instruction::Status => {
                debug!("Instruction: Status");

                let listeners: Vec<String> =
                    self.swarm.listeners().map(|f| f.to_string()).collect();
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

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "GistitNetworkEvent")]
struct GistitNetworkBehaviour {
    req_res: RequestResponse<GistitExchangeCodec>,
    kademlia: Kademlia<MemoryStore>,
    identify: Identify,
    ping: Ping,
    relay: relay::Relay,
    autonat: autonat::Behaviour,
}

#[derive(Debug)]
enum GistitNetworkEvent {
    RequestResponse(RequestResponseEvent<GistitRequest, GistitResponse>),
    Kademlia(KademliaEvent),
    Identify(IdentifyEvent),
    Ping(PingEvent),
    Relay(relay::Event),
    Autonat(autonat::Event),
}

impl From<RequestResponseEvent<GistitRequest, GistitResponse>> for GistitNetworkEvent {
    fn from(event: RequestResponseEvent<GistitRequest, GistitResponse>) -> Self {
        GistitNetworkEvent::RequestResponse(event)
    }
}

impl From<KademliaEvent> for GistitNetworkEvent {
    fn from(event: KademliaEvent) -> Self {
        GistitNetworkEvent::Kademlia(event)
    }
}

impl From<PingEvent> for GistitNetworkEvent {
    fn from(event: PingEvent) -> Self {
        GistitNetworkEvent::Ping(event)
    }
}

impl From<IdentifyEvent> for GistitNetworkEvent {
    fn from(event: IdentifyEvent) -> Self {
        GistitNetworkEvent::Identify(event)
    }
}

impl From<relay::Event> for GistitNetworkEvent {
    fn from(event: relay::Event) -> Self {
        GistitNetworkEvent::Relay(event)
    }
}

impl From<autonat::Event> for GistitNetworkEvent {
    fn from(event: autonat::Event) -> Self {
        GistitNetworkEvent::Autonat(event)
    }
}

#[derive(Debug, Clone)]
pub struct GistitExchangeProtocol;

impl ProtocolName for GistitExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/gistit/1"
    }
}

#[derive(Clone)]
pub struct GistitExchangeCodec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GistitRequest(Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GistitResponse(Vec<u8>);

#[async_trait]
impl RequestResponseCodec for GistitExchangeCodec {
    type Protocol = GistitExchangeProtocol;
    type Request = GistitRequest;
    type Response = GistitResponse;

    async fn read_request<T: Send + Unpin + AsyncRead>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> tokio::io::Result<Self::Request> {
        // FIXME: Export all consts params
        let bytes = read_length_prefixed(io, 50_000).await?;

        if bytes.is_empty() {
            Err(tokio::io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(GistitRequest(bytes))
        }
    }

    async fn read_response<T: Send + Unpin + AsyncRead>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> tokio::io::Result<Self::Response> {
        // FIXME: Export all consts params
        let bytes = read_length_prefixed(io, 50_000).await?;

        if bytes.is_empty() {
            Err(tokio::io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(GistitResponse(bytes))
        }
    }

    async fn write_request<T: Send + Unpin + AsyncWrite>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        GistitRequest(data): Self::Request,
    ) -> tokio::io::Result<()> {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        GistitResponse(data): Self::Response,
    ) -> tokio::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
