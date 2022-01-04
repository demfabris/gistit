//! The network module
#![allow(clippy::missing_errors_doc)]

use std::iter::once;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use unchecked_unwrap::UncheckedUnwrap;

use async_trait::async_trait;
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::core::{PeerId, ProtocolName};
use libp2p::futures::{self, future::poll_fn};
use libp2p::futures::{AsyncRead, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::identity::Keypair;
use libp2p::multiaddr::multiaddr;
use libp2p::request_response::{
    ProtocolSupport, RequestId, RequestResponse, RequestResponseCodec, RequestResponseConfig,
    RequestResponseEvent, RequestResponseMessage, ResponseChannel,
};
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmBuilder, SwarmEvent};
use libp2p::{development_transport, Swarm, Transport};
use libp2p::{identity, Multiaddr};
use notify::{raw_watcher, RawEvent as FsEvent, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{Error, Result};

pub struct NetworkConfig {
    event_loop_config: EventLoopConfig,
    watcher: RecommendedWatcher,
}

impl NetworkConfig {
    pub fn new(seed: &str, local_multiaddr: Multiaddr, runtime_dir: &Path) -> Result<Self> {
        let keypair = if seed == "none" {
            identity::Keypair::generate_ed25519()
        } else {
            let mut bytes: Vec<u8> = seed.as_bytes().to_vec();
            bytes.resize_with(32, || 0);
            let mut bytes: [u8; 32] = bytes.try_into().unwrap();

            let ed25519_secret = identity::ed25519::SecretKey::from_bytes(&mut bytes).unwrap();
            identity::Keypair::Ed25519(ed25519_secret.into())
        };
        let FsWatcher {
            watcher,
            channel: (_, watcher_rx),
        } = FsWatcher::new(runtime_dir)?;

        let event_loop_config = EventLoopConfig {
            peer_id: PeerId::from(keypair.public()),
            keypair,
            local_multiaddr,
            watcher_rx: Arc::new(Mutex::new(watcher_rx)),
        };

        Ok(Self {
            event_loop_config,
            watcher,
        })
    }

    pub async fn into_node(self) -> Result<NetworkNode> {
        let event_loop = EventLoop::from_config(self.event_loop_config)
            .await?
            .prepare()
            .await;

        Ok(NetworkNode {
            event_loop,
            __watcher: self.watcher,
        })
    }
}

pub struct NetworkNode {
    event_loop: EventLoop,
    __watcher: RecommendedWatcher,
}

impl NetworkNode {
    pub async fn run(self) {
        self.event_loop.run().await;
    }
}

struct FsWatcher {
    watcher: RecommendedWatcher,
    channel: (Sender<FsEvent>, Receiver<FsEvent>),
}

impl FsWatcher {
    fn new(cache_dir: &Path) -> Result<Self> {
        let (watcher_tx, watcher_rx) = channel();

        let mut watcher = raw_watcher(watcher_tx.clone())?;
        watcher.watch(cache_dir, RecursiveMode::Recursive)?;

        Ok(Self {
            watcher,
            channel: (watcher_tx, watcher_rx),
        })
    }
}

#[derive(Clone)]
pub struct EventLoopConfig {
    peer_id: PeerId,
    keypair: Keypair,
    local_multiaddr: Multiaddr,
    watcher_rx: Arc<Mutex<Receiver<FsEvent>>>,
}

/// The main event loop
pub struct EventLoop {
    /// p2p Swarm, acts like a receiver
    swarm: Swarm<RequestResponse<GistitExchangeCodec>>,
    /// Fs events watcher
    watcher_rx: Arc<Mutex<Receiver<FsEvent>>>,
}

impl EventLoop {
    async fn from_config(config: EventLoopConfig) -> Result<Self> {
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
        swarm.listen_on(config.local_multiaddr).unwrap(); //TODO: dont panic

        Ok(Self {
            swarm,
            watcher_rx: config.watcher_rx,
        })
    }

    async fn prepare(mut self) -> Self {
        while let SwarmEvent::NewListenAddr { address, .. } = self.swarm.select_next_some().await {
            println!("{:?}", address);
        }
        self
    }

    async fn run(mut self) {
        // TODO: routine to check current peers
        loop {
            tokio::select! {
                swarm_event = self.swarm.next() => self.handle_swarm_event(
                    swarm_event.expect("some event")).await,

                fs_event = poll_fn(|_| {
                    let watcher = self.watcher_rx.lock().expect("to lock");
                    futures::task::Poll::Ready(watcher.recv())
                }) => self.handle_fs_event(fs_event.expect("to receive fs event")).await
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

    async fn handle_fs_event(&mut self, fs_event: FsEvent) {
        match fs_event {
            FsEvent {
                path: Some(peer),
                op: Ok(notify::op::Op::CREATE),
                ..
            } => {
                let addr_str = peer.file_name().expect("valid").to_string_lossy();
                println!("{:?}", addr_str);
                let peer_multiaddr =
                    std::str::from_utf8(base64::decode(addr_str.as_ref()).unwrap().as_slice())
                        .unwrap()
                        .parse::<Multiaddr>()
                        .unwrap();
                self.swarm.dial(peer_multiaddr).unwrap();
                println!("Connected to peer {:?}", addr_str);
            }
            FsEvent {
                path: Some(peer),
                op: Ok(notify::op::Op::REMOVE),
                ..
            } => {
                println!("removed");
            }
            _ => (),
        }
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
