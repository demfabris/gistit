//! The network module
#![allow(clippy::missing_errors_doc)]

use std::iter::once;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};

use async_trait::async_trait;
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::core::{PeerId, ProtocolName};
use libp2p::futures::{self, future::poll_fn};
use libp2p::futures::{AsyncRead, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::identity::Keypair;
use libp2p::multiaddr::multiaddr;
use libp2p::request_response::{
    ProtocolSupport, RequestResponse, RequestResponseCodec, RequestResponseConfig,
    RequestResponseEvent,
};
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmBuilder, SwarmEvent};
use libp2p::{development_transport, Swarm};
use libp2p::{identity, Multiaddr};

use lib_gistit::ipc::Bridge;

use crate::Result;

pub struct NetworkConfig {
    peer_id: PeerId,
    keypair: Keypair,
    local_multiaddr: Multiaddr,
    bridge: Bridge,
}

impl NetworkConfig {
    pub fn new(
        seed: &str,
        local_multiaddr: Multiaddr,
        runtime_dir: &Path,
        bridge: Bridge,
    ) -> Result<Self> {
        let mut bytes: Vec<u8> = seed.as_bytes().to_vec();
        bytes.resize_with(32, || 0);
        let mut bytes: [u8; 32] = bytes.try_into().unwrap();

        let ed25519_secret = identity::ed25519::SecretKey::from_bytes(&mut bytes).unwrap();
        let keypair = identity::Keypair::Ed25519(ed25519_secret.into());

        let peer_id = PeerId::from(keypair.public());

        Ok(Self {
            peer_id,
            keypair,
            local_multiaddr,
            bridge,
        })
    }

    pub async fn apply(self) -> Result<NetworkNode> {
        NetworkNode::new(self).await
    }
}

/// The main event loop
pub struct NetworkNode {
    /// p2p Swarm, acts like a receiver
    swarm: Swarm<RequestResponse<GistitExchangeCodec>>,
    bridge: Bridge,
}

impl NetworkNode {
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        let mut swarm = SwarmBuilder::new(
            development_transport(config.keypair).await.unwrap(), // TODO: dont panic
            RequestResponse::new(
                GistitExchangeCodec,
                once((GistitExchangeProtocol, ProtocolSupport::Full)),
                RequestResponseConfig::default(),
            ),
            config.peer_id,
        )
        .build();

        println!("Listening on {:?}", config.local_multiaddr);
        swarm.listen_on(config.local_multiaddr).unwrap(); //TODO: dont panic

        Ok(Self {
            swarm,
            bridge: config.bridge,
        })
    }

    pub fn peer_id(&self) -> Vec<u8> {
        self.swarm.local_peer_id().to_bytes()
    }

    pub async fn run(mut self) -> Result<()> {
        // TODO: routine to check current peers
        let mut buf = Vec::new();
        let mut buf = tokio::io::ReadBuf::new(&mut buf);
        loop {
            tokio::select! {
                swarm_event = self.swarm.next() => self.handle_swarm_event(
                    swarm_event.expect("some event")).await,

                bridge_event = poll_fn(|ctx| {
                    self.bridge.sock.poll_recv(ctx, &mut buf)
                }) => {
                    println!("{:?}", bridge_event.unwrap());
                },

                // fs_event = poll_fn(|_| {
                //     let watcher_rx = self.fs_watcher.channel.1.clone();
                //     let watcher_rx = watcher_rx.lock().expect("to lock");
                //     futures::task::Poll::Ready(watcher_rx.recv())
                // }) => self.handle_fs_event(fs_event.expect("to receive fs event")).await?
            }
        }
    }

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<
            RequestResponseEvent<GistitRequest, GistitResponse>,
            ProtocolsHandlerUpgrErr<std::io::Error>,
        >,
    ) {
        println!("{:?}", event);
    }

    async fn handle_bridge_event(&mut self) -> Result<()> {
        todo!()
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

#[must_use]
pub fn ipv4_to_multiaddr(addr: Ipv4Addr, port: u16) -> Multiaddr {
    multiaddr!(Ip4(addr), Tcp(port))
}
