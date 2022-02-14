//
//   ________.__          __  .__  __
//  /  _____/|__| _______/  |_|__|/  |_
// /   \  ___|  |/  ___/\   __\  \   __\
// \    \_\  \  |\___ \  |  | |  ||  |
//  \______  /__/____  > |__| |__||__|
//         \/        \/
//
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(
    test,
    allow(
        unused,
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::dbg_macro,
        clippy::unwrap_used,
        clippy::missing_docs_in_private_items,
    )
)]
//! This is a simple crate to handle the inter process comms for gistit-daemon and gistit-cli
//! TODO: Missing TCP socket implementation

use std::fs::{metadata, remove_file};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::net::UnixDatagram;

use gistit_proto::bytes::BytesMut;
use gistit_proto::ipc;
use gistit_proto::prost::{self, Message};

pub type Result<T> = std::result::Result<T, Error>;

const NAMED_SOCKET_0: &str = "gistit-0";
const NAMED_SOCKET_1: &str = "gistit-1";

const READBUF_SIZE: usize = 60_000; // A bit bigger than 50kb because encoding
const CONNECT_TIMEOUT_SECS: u64 = 3;

pub trait SockEnd {}

#[derive(Debug)]
pub struct Server;
impl SockEnd for Server {}

#[derive(Debug)]
pub struct Client;
impl SockEnd for Client {}

#[derive(Debug)]
pub struct Bridge<T: SockEnd> {
    pub sock_0: UnixDatagram,
    pub sock_1: UnixDatagram,
    base: PathBuf,
    __marker_t: PhantomData<T>,
}

/// Recv from [`NAMED_SOCKET_0`] and send to [`NAMED_SOCKET_1`]
/// The owner of `sock_0`
///
/// # Errors
///
/// Fails if can't spawn a named socket
pub fn server(base: &Path) -> Result<Bridge<Server>> {
    let sockpath_0 = &base.join(NAMED_SOCKET_0);

    if metadata(sockpath_0).is_ok() {
        remove_file(sockpath_0)?;
    }

    log::trace!("Bind sock_0 (server) at {:?}", sockpath_0);
    let sock_0 = UnixDatagram::bind(sockpath_0)?;

    Ok(Bridge {
        sock_0,
        sock_1: UnixDatagram::unbound()?,
        base: base.to_path_buf(),
        __marker_t: PhantomData,
    })
}

/// Recv from [`NAMED_SOCKET_1`] and send to [`NAMED_SOCKET_0`]
/// The owner of `sock_1`
///
/// # Errors
///
/// Fails if can't spawn a named socket
pub fn client(base: &Path) -> Result<Bridge<Client>> {
    let sockpath_1 = &base.join(NAMED_SOCKET_1);

    if metadata(sockpath_1).is_ok() {
        remove_file(sockpath_1)?;
    }

    log::trace!("Bind sock_1 (client) at {:?}", sockpath_1);
    let sock_1 = UnixDatagram::bind(sockpath_1)?;

    Ok(Bridge {
        sock_0: UnixDatagram::unbound()?,
        sock_1,
        base: base.to_path_buf(),
        __marker_t: PhantomData,
    })
}

fn __alive(base: &Path, dgram: &UnixDatagram, sock_name: &str) -> bool {
    !matches!(dgram.connect(base.join(sock_name)), Err(_))
}

fn __connect_blocking(base: &Path, dgram: &UnixDatagram, sock_name: &str) -> Result<()> {
    let earlier = Instant::now();
    while let Err(err) = dgram.connect(base.join(sock_name)) {
        if Instant::now().duration_since(earlier).as_secs() > CONNECT_TIMEOUT_SECS {
            return Err(err.into());
        }
    }

    log::trace!("Connecting to {:?}", sock_name);
    Ok(())
}

impl Bridge<Server> {
    pub fn alive(&self) -> bool {
        __alive(&self.base, &self.sock_1, NAMED_SOCKET_1)
    }

    /// Connect to the other end
    ///
    /// # Errors
    ///
    /// Inherits errors of [`__connect_blocking`]
    pub fn connect_blocking(&mut self) -> Result<()> {
        __connect_blocking(&self.base, &self.sock_1, NAMED_SOCKET_1)
    }

    /// Send bincode serialized data through the pipe
    ///
    /// # Errors
    ///
    /// Fails if the socket is not alive
    pub async fn send(&self, instruction: ipc::Instruction) -> Result<()> {
        let mut buf = BytesMut::with_capacity(READBUF_SIZE);
        instruction.encode(&mut buf)?;
        log::trace!("Sending to client {} bytes", buf.len());
        self.sock_1.send(&buf).await?;
        Ok(())
    }

    /// Attempts to receive serialized data from the pipe
    ///
    /// # Errors
    ///
    /// Fails if the socket is not alive
    pub async fn recv(&self) -> Result<ipc::Instruction> {
        let mut buf = vec![0u8; READBUF_SIZE];
        let read = self.sock_0.recv(&mut buf).await?;
        buf.truncate(read);
        let target = ipc::Instruction::decode(&*buf)?;
        Ok(target)
    }
}

impl Bridge<Client> {
    pub fn alive(&self) -> bool {
        __alive(&self.base, &self.sock_0, NAMED_SOCKET_0)
    }

    /// Connect to the other end
    ///
    /// # Errors
    ///
    /// Inherits errors of [`__connect_blocking`]
    pub fn connect_blocking(&mut self) -> Result<()> {
        __connect_blocking(&self.base, &self.sock_0, NAMED_SOCKET_0)
    }

    /// Send bincode serialized data through the pipe
    ///
    /// # Errors
    ///
    /// Fails if the socket is not alive
    pub async fn send(&self, instruction: ipc::Instruction) -> Result<()> {
        let mut buf = BytesMut::with_capacity(READBUF_SIZE);
        instruction.encode(&mut buf)?;
        log::trace!("Sending to server {} bytes", buf.len());
        self.sock_0.send(&*buf).await?;
        Ok(())
    }

    /// Attempts to receive serialized data from the pipe
    ///
    /// # Errors
    ///
    /// Fails if the socket is not alive
    pub async fn recv(&self) -> Result<ipc::Instruction> {
        let mut buf = vec![0u8; READBUF_SIZE];
        let read = self.sock_1.recv(&mut buf).await?;
        buf.truncate(read);
        let target = ipc::Instruction::decode(&*buf)?;
        Ok(target)
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(std::io::Error),
    Decode(prost::DecodeError),
    Encode(prost::EncodeError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::IO(err),
        }
    }
}

impl From<prost::DecodeError> for Error {
    fn from(err: prost::DecodeError) -> Self {
        Self {
            kind: ErrorKind::Decode(err),
        }
    }
}

impl From<prost::EncodeError> for Error {
    fn from(err: prost::EncodeError) -> Self {
        Self {
            kind: ErrorKind::Encode(err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "gistit ipc error"
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gistit ipc error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use std::sync::Arc;

    pub fn test_instruction_1() -> ipc::Instruction {
        let mut instruction = ipc::Instruction::default();
        instruction.kind = Some(ipc::instruction::Kind::Status(ipc::instruction::Status {}));
        instruction
    }

    pub fn test_instruction_2() -> ipc::Instruction {
        let mut instruction = ipc::Instruction::default();
        instruction.kind = Some(ipc::instruction::Kind::Status(ipc::instruction::Status {}));
        instruction
    }

    #[tokio::test]
    async fn ipc_named_socket_spawn() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let _ = server(&tmp).unwrap();
        let _ = client(&tmp).unwrap();

        assert!(tmp.child("gistit-0").exists());
        assert!(tmp.child("gistit-1").exists());
    }

    #[tokio::test]
    async fn ipc_socket_spawn_is_alive() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let server = server(&tmp).unwrap();
        let client = client(&tmp).unwrap();

        assert!(server.alive());
        assert!(client.alive());
    }

    #[tokio::test]
    async fn ipc_socket_server_recv_traffic() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let server = server(&tmp).unwrap();
        let mut client = client(&tmp).unwrap();

        client.connect_blocking().unwrap();

        client.send(test_instruction_1()).await.unwrap();
        client.send(test_instruction_2()).await.unwrap();

        assert_eq!(server.recv().await.unwrap(), test_instruction_1());
        assert_eq!(server.recv().await.unwrap(), test_instruction_2());
    }

    #[tokio::test]
    async fn ipc_socket_client_recv_traffic() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut server = server(&tmp).unwrap();
        let client = client(&tmp).unwrap();

        server.connect_blocking().unwrap();

        server.send(test_instruction_1()).await.unwrap();
        server.send(test_instruction_2()).await.unwrap();

        assert_eq!(client.recv().await.unwrap(), test_instruction_1());
        assert_eq!(client.recv().await.unwrap(), test_instruction_2());
    }

    #[tokio::test]
    async fn ipc_socket_alternate_traffic() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut server = server(&tmp).unwrap();
        let mut client = client(&tmp).unwrap();

        client.connect_blocking().unwrap();
        server.connect_blocking().unwrap();

        client.send(test_instruction_1()).await.unwrap();
        client.send(test_instruction_2()).await.unwrap();

        server.send(test_instruction_1()).await.unwrap();
        server.send(test_instruction_2()).await.unwrap();

        assert_eq!(client.recv().await.unwrap(), test_instruction_1());
        assert_eq!(server.recv().await.unwrap(), test_instruction_1());
        assert_eq!(client.recv().await.unwrap(), test_instruction_2());
        assert_eq!(server.recv().await.unwrap(), test_instruction_2());
    }

    #[tokio::test]
    async fn ipc_socket_alternate_traffic_rerun() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut server = server(&tmp).unwrap();
        let mut client = client(&tmp).unwrap();

        client.connect_blocking().unwrap();
        server.connect_blocking().unwrap();

        client.send(test_instruction_1()).await.unwrap();
        client.send(test_instruction_2()).await.unwrap();

        server.send(test_instruction_1()).await.unwrap();
        server.send(test_instruction_2()).await.unwrap();

        assert_eq!(client.recv().await.unwrap(), test_instruction_1());
        assert_eq!(server.recv().await.unwrap(), test_instruction_1());
        assert_eq!(client.recv().await.unwrap(), test_instruction_2());
        assert_eq!(server.recv().await.unwrap(), test_instruction_2());

        client.send(test_instruction_1()).await.unwrap();
        client.send(test_instruction_2()).await.unwrap();

        server.send(test_instruction_1()).await.unwrap();
        server.send(test_instruction_2()).await.unwrap();

        assert_eq!(client.recv().await.unwrap(), test_instruction_1());
        assert_eq!(server.recv().await.unwrap(), test_instruction_1());
        assert_eq!(client.recv().await.unwrap(), test_instruction_2());
        assert_eq!(server.recv().await.unwrap(), test_instruction_2());
    }

    #[tokio::test]
    async fn ipc_socket_traffic_under_load() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut server = server(&tmp).unwrap();
        let mut client = client(&tmp).unwrap();

        client.connect_blocking().unwrap();
        server.connect_blocking().unwrap();

        let server = Arc::new(server);
        let client = Arc::new(client);

        for _ in 0..8 {
            let s = server.clone();
            let c = client.clone();

            tokio::spawn(async move {
                loop {
                    c.send(test_instruction_1()).await.unwrap();
                    c.send(test_instruction_2()).await.unwrap();

                    s.send(test_instruction_1()).await.unwrap();
                    s.send(test_instruction_2()).await.unwrap();
                }
            });

            assert_eq!(client.recv().await.unwrap(), test_instruction_1());
            assert_eq!(server.recv().await.unwrap(), test_instruction_1());
            assert_eq!(client.recv().await.unwrap(), test_instruction_2());
            assert_eq!(server.recv().await.unwrap(), test_instruction_2());
        }
    }
}
