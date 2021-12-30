//! The network module
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::missing_errors_doc)]

use std::iter::once;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use async_trait::async_trait;
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::core::{PeerId, ProtocolName};
use libp2p::futures::Stream;
use libp2p::futures::{self, future::poll_fn};
use libp2p::futures::{AsyncRead, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::request_response::{
    ProtocolSupport, RequestId, RequestResponse, RequestResponseCodec, RequestResponseConfig,
    RequestResponseEvent, RequestResponseMessage, ResponseChannel,
};
use libp2p::swarm::{ProtocolsHandlerUpgrErr, SwarmBuilder, SwarmEvent};
use libp2p::{development_transport, Swarm, Transport};
use libp2p::{identity, Multiaddr};
use notify::{raw_watcher, RawEvent as FsEvent, RecommendedWatcher, RecursiveMode, Watcher};

use crate::errors::internal::InternalError;
use crate::{Error, Result};

pub struct NetworkDaemon {
    event_loop: EventLoop,
    watcher: RecommendedWatcher,
    persist: bool,
}

impl NetworkDaemon {
    /// # Errors
    ///
    /// asd
    pub async fn new(password: &str, cache_dir: &Path) -> Result<Self> {
        let ed25519_keypair = if password == "none" {
            identity::Keypair::generate_ed25519()
        } else {
            let mut bytes: Vec<u8> = password.as_bytes().to_vec();
            bytes.resize_with(32, || 0);
            let mut bytes: [u8; 32] = bytes.try_into().unwrap();

            let ed25519_secret = identity::ed25519::SecretKey::from_bytes(&mut bytes).unwrap();
            identity::Keypair::Ed25519(ed25519_secret.into())
        };

        println!("{:?}", ed25519_keypair.public());
        let peer_id = PeerId::from(ed25519_keypair.public());

        let swarm = SwarmBuilder::new(
            development_transport(ed25519_keypair).await.unwrap(),
            RequestResponse::new(
                GistitExchangeCodec,
                once((GistitExchangeProtocol, ProtocolSupport::Full)),
                RequestResponseConfig::default(),
            ),
            peer_id,
        )
        .build();

        let FsWatcher {
            watcher,
            channel: (_, watcher_rx),
        } = FsWatcher::new(cache_dir)?;

        Ok(Self {
            event_loop: EventLoop::new(swarm, Mutex::new(watcher_rx)).await?,
            watcher,
            persist: false,
        })
    }

    pub const fn persist(mut self, yes: bool) -> Self {
        self.persist = yes;
        self
    }

    pub fn listen(mut self, address_port: &str) -> Self {
        let (addr, port) = address_port.split_once(':').expect("to contain :"); // FIXME: dont panic
        println!("addr {} port {}", addr, port);
        let multiaddr: Multiaddr = format!("/ip4/{}/tcp/{}", addr, port).parse().unwrap();
        self.event_loop.swarm.listen_on(multiaddr).unwrap();
        self
    }

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

/// The main event loop
pub struct EventLoop {
    /// p2p Swarm, acts like a receiver
    swarm: Swarm<RequestResponse<GistitExchangeCodec>>,
    /// Fs events watcher
    watcher_rx: Mutex<Receiver<FsEvent>>,
}

impl EventLoop {
    async fn new(
        swarm: Swarm<RequestResponse<GistitExchangeCodec>>,
        watcher_rx: Mutex<Receiver<FsEvent>>,
    ) -> Result<Self> {
        Ok(Self { swarm, watcher_rx })
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                swarm_event = self.swarm.next() => self.handle_swarm_event(swarm_event.expect("some event")).await,

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
                let addr_str = peer.as_os_str().to_string_lossy();
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

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Self {
        Self::Internal(InternalError::Other(err.to_string()))
    }
}
