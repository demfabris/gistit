//! The network module
#![allow(clippy::missing_errors_doc)]

use std::collections::HashMap;
use std::io;
use std::iter::once;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use libp2p::core::either::EitherError;
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::core::{PeerId, ProtocolName};
use libp2p::futures::channel::oneshot;
use libp2p::futures::{AsyncRead, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::identity::Keypair;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{GetProvidersOk, Kademlia, KademliaConfig, KademliaEvent, QueryId, QueryResult};
use libp2p::multiaddr::multiaddr;
use libp2p::request_response::{
    ProtocolSupport, RequestResponse, RequestResponseCodec, RequestResponseConfig,
    RequestResponseEvent,
};
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmEvent};
use libp2p::{development_transport, Swarm};
use libp2p::{identity, Multiaddr, NetworkBehaviour};

use lib_gistit::ipc::{self, Bridge, Instruction, Server, ServerResponse};

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
    // pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    // pending_start_providing: HashMap<QueryId, oneshot::Sender<()>>,
    // pending_get_providers: HashMap<QueryId, oneshot::Sender<HashSet<PeerId>>>,
    // pending_request_file:
    //     HashMap<RequestId, oneshot::Sender<Result<String, Box<dyn Error + Send>>>>,
}

const BOOTNODES: [&'static str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

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

            let bootaddr = Multiaddr::from_str("/dnsaddr/bootstrap.libp2p.io")?;
            for peer in &BOOTNODES {
                behaviour.add_address(
                    &PeerId::from_str(peer).expect("peer id to be valid"),
                    bootaddr.clone(),
                );
            }

            behaviour
        };

        let swarm = Swarm::new(
            development_transport(config.keypair)
                .await
                .expect("start p2p transport"),
            GistitNetworkBehaviour { req_res, kademlia },
            config.peer_id,
        );

        let bridge = ipc::server(&config.runtime_dir)?;

        Ok(Self { swarm, bridge })
    }

    pub fn peer_id(&self) -> String {
        self.swarm.local_peer_id().to_base58()
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                swarm_event = self.swarm.next() => self.handle_swarm_event(
                    swarm_event.expect("to recv swarm event")).await?,

                bridge_event = self.bridge.recv() => self.handle_bridge_event(bridge_event?).await
            }
        }
    }

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<
            GistitNetworkEvent,
            EitherError<ProtocolsHandlerUpgrErr<io::Error>, io::Error>,
        >,
    ) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on: {:?}", address);
                let peer_id = self.swarm.local_peer_id().to_string();
                self.bridge.connect_blocking()?;
                self.bridge
                    .send(Instruction::Response(ServerResponse::PeerId(peer_id)))
                    .await?;
            }
            _ => (),
        }
        Ok(())
    }

    async fn handle_bridge_event(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Listen { host, port } => {
                println!("Listening on {:?}:{:?}", host, port);
                let addr = multiaddr!(Ip4(host), Tcp(port));
                self.swarm.listen_on(addr).expect("to listen to addr");
            }
            Instruction::Dial { raw_address } => {
                let addr: Multiaddr = raw_address.parse().expect("to be valid multiaddr");
                println!("{:?}", addr);
            }
            Instruction::Shutdown => {
                println!("Exiting");
                drop(self);
                std::process::exit(0);
            }
            Instruction::File(data) => {
                println!("{:?}", data);
            }
            _ => (),
        }
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "GistitNetworkEvent")]
struct GistitNetworkBehaviour {
    req_res: RequestResponse<GistitExchangeCodec>,
    kademlia: Kademlia<MemoryStore>,
}

#[derive(Debug)]
enum GistitNetworkEvent {
    RequestResponse(RequestResponseEvent<GistitRequest, GistitResponse>),
    Kademlia(KademliaEvent),
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
