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
use libp2p::{development_transport, Swarm};
use libp2p::{identity, Multiaddr};
use notify::{raw_watcher, RawEvent as FsEvent, RecommendedWatcher, RecursiveMode, Watcher};

use crate::errors::internal::InternalError;
use crate::{Error, Result};

pub struct NetworkDaemon {
    client: Client,
    event_loop: EventLoop,
    _watcher: RecommendedWatcher,
}

impl NetworkDaemon {
    /// # Errors
    ///
    /// asd
    pub async fn new(secret: &str, host_dir: &Path) -> Result<Self> {
        // TODO: improve the keypair
        let ed25519_keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(ed25519_keypair.public());

        // The command interface
        let (cmd_tx, cmd_rx) = futures::channel::mpsc::channel(0);
        let (cli_event_tx, cli_event_rx) = futures::channel::mpsc::channel(0);
        let client = Client { sender: cmd_tx };

        let mut swarm = SwarmBuilder::new(
            development_transport(ed25519_keypair).await.unwrap(),
            RequestResponse::new(
                GistitExchangeCodec,
                once((GistitExchangeProtocol, ProtocolSupport::Full)),
                RequestResponseConfig::default(),
            ),
            peer_id,
        )
        .build();
        swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .unwrap();

        let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();
        let mut watcher = raw_watcher(watcher_tx)?;
        watcher.watch(host_dir, RecursiveMode::Recursive)?;
        let watcher_rx = Mutex::new(watcher_rx);

        Ok(Self {
            client,
            event_loop: EventLoop::new(swarm, cmd_rx, cli_event_tx, watcher_rx).await?,
            _watcher: watcher,
        })
    }

    pub async fn run(self) {
        self.event_loop.run().await;
    }
}

/// The main event loop
pub struct EventLoop {
    /// p2p Swarm, acts like a receiver
    swarm: Swarm<RequestResponse<GistitExchangeCodec>>,
    /// The command instructions receiver
    cmd_rx: futures::channel::mpsc::Receiver<Command>,
    /// The response event
    cli_event_tx: futures::channel::mpsc::Sender<CliEventResponse>,
    /// Fs events watcher
    watcher_rx: Mutex<Receiver<FsEvent>>,
}

impl EventLoop {
    async fn new(
        swarm: Swarm<RequestResponse<GistitExchangeCodec>>,
        cmd_rx: futures::channel::mpsc::Receiver<Command>,
        cli_event_tx: futures::channel::mpsc::Sender<CliEventResponse>,
        watcher_rx: Mutex<Receiver<FsEvent>>,
    ) -> Result<Self> {
        Ok(Self {
            swarm,
            cmd_rx,
            cli_event_tx,
            watcher_rx,
        })
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                // Swarm events
                swarm_event = self.swarm.next() => self.handle_swarm_event(swarm_event.expect("some event")).await,

                fs_event = poll_fn(|_| {
                    let watcher = self.watcher_rx.lock().expect("to lock");
                    futures::task::Poll::Ready(watcher.recv())
                }) => self.handle_fs_event(fs_event.expect("to receive fs event")).await
            }
        }
    }

    async fn handle_command(&mut self, cmd: Command) {
        todo!()
    }

    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<
            RequestResponseEvent<GistitRequest, GistitResponse>,
            ProtocolsHandlerUpgrErr<std::io::Error>,
        >,
    ) {
        todo!()
    }

    async fn handle_fs_event(&mut self, fs_event: FsEvent) {
        match fs_event {
            FsEvent {
                path: Some(peer),
                op: Ok(notify::op::Op::CREATE),
                ..
            } => {
                println!("Connected to peer {:?}", peer);
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

#[derive(Clone)]
pub struct Client {
    sender: futures::channel::mpsc::Sender<Command>,
}

impl Client {
    pub fn listen(&mut self, addr: Multiaddr) -> Result<()> {
        let (tx, mut rx) = futures::channel::oneshot::channel::<Command>();
        self.sender
            .try_send(Command::Listen { addr, tx })
            .expect("To send listen command");
        rx.try_recv().expect("To receive listen command");
        Ok(())
    }

    pub fn dial(&mut self, peer_id: PeerId, peer_addr: Multiaddr) -> Result<()> {
        let (sender, mut recv) = futures::channel::oneshot::channel::<Command>();
        self.sender
            .try_send(Command::Dial)
            .expect("To send dial command");
        recv.try_recv().expect("To receive dial command");
        Ok(())
    }
}

pub enum Command {
    Listen {
        addr: Multiaddr,
        tx: futures::channel::oneshot::Sender<Command>,
    },
    Dial,
}

pub struct CliEventResponse;

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
