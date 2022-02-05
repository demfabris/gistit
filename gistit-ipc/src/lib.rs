//! This is a simple crate to a communication interface for gistit-daemon and gistit-cli
//! Missing TCP socket implementation

use std::fs::{metadata, remove_file};
use std::marker::PhantomData;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::net::UnixDatagram;

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

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
/// The owner of sock_0
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
/// The owner of sock_1
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

    pub fn connect_blocking(&mut self) -> Result<()> {
        __connect_blocking(&self.base, &self.sock_1, NAMED_SOCKET_1)
    }

    pub async fn send(&self, instruction: Instruction) -> Result<()> {
        let encoded = serialize(&instruction).unwrap();
        log::trace!("Sending to client {} bytes", encoded.len());
        self.sock_1.send(&encoded).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<Instruction> {
        let mut buf = vec![0u8; READBUF_SIZE];
        self.sock_0.recv(&mut buf).await?;
        let target = deserialize(&buf).unwrap();
        Ok(target)
    }
}

impl Bridge<Client> {
    pub fn alive(&self) -> bool {
        __alive(&self.base, &self.sock_0, NAMED_SOCKET_0)
    }

    pub fn connect_blocking(&mut self) -> Result<()> {
        __connect_blocking(&self.base, &self.sock_0, NAMED_SOCKET_0)
    }

    pub async fn send(&self, instruction: Instruction) -> Result<()> {
        let encoded = serialize(&instruction).unwrap();
        log::trace!("Sending to server {} bytes", encoded.len());
        self.sock_0.send(&encoded).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<Instruction> {
        let mut buf = vec![0u8; READBUF_SIZE];
        self.sock_1.recv(&mut buf).await?;
        let target = deserialize(&buf).unwrap();
        Ok(target)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Instruction {
    Listen {
        host: Ipv4Addr,
        port: u16,
    },
    Dial {
        peer_id: String,
    },
    Provide {
        hash: String,
        data: Vec<u8>,
    },
    Get {
        hash: String,
    },
    Status,
    Shutdown,
    // Daemon responses
    Response(ServerResponse),

    #[cfg(test)]
    TestInstructionOne,
    #[cfg(test)]
    TestInstructionTwo,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ServerResponse {
    PeerId(String),
    Status(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::IO(err),
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

        client.send(Instruction::TestInstructionOne).await.unwrap();
        client.send(Instruction::TestInstructionTwo).await.unwrap();

        assert_eq!(
            server.recv().await.unwrap(),
            Instruction::TestInstructionOne
        );
        assert_eq!(
            server.recv().await.unwrap(),
            Instruction::TestInstructionTwo
        );
    }

    #[tokio::test]
    async fn ipc_socket_client_recv_traffic() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut server = server(&tmp).unwrap();
        let client = client(&tmp).unwrap();

        server.connect_blocking().unwrap();

        server.send(Instruction::TestInstructionOne).await.unwrap();
        server.send(Instruction::TestInstructionTwo).await.unwrap();

        assert_eq!(
            client.recv().await.unwrap(),
            Instruction::TestInstructionOne
        );
        assert_eq!(
            client.recv().await.unwrap(),
            Instruction::TestInstructionTwo
        );
    }

    #[tokio::test]
    async fn ipc_socket_alternate_traffic() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut server = server(&tmp).unwrap();
        let mut client = client(&tmp).unwrap();

        client.connect_blocking().unwrap();
        server.connect_blocking().unwrap();

        client.send(Instruction::TestInstructionOne).await.unwrap();
        client.send(Instruction::TestInstructionTwo).await.unwrap();

        server.send(Instruction::TestInstructionOne).await.unwrap();
        server.send(Instruction::TestInstructionTwo).await.unwrap();

        assert_eq!(
            client.recv().await.unwrap(),
            Instruction::TestInstructionOne
        );
        assert_eq!(
            server.recv().await.unwrap(),
            Instruction::TestInstructionOne
        );
        assert_eq!(
            client.recv().await.unwrap(),
            Instruction::TestInstructionTwo
        );
        assert_eq!(
            server.recv().await.unwrap(),
            Instruction::TestInstructionTwo
        );
    }

    #[tokio::test]
    async fn ipc_socket_alternate_traffic_rerun() {
        let tmp = assert_fs::TempDir::new().unwrap();
        let mut server = server(&tmp).unwrap();
        let mut client = client(&tmp).unwrap();

        client.connect_blocking().unwrap();
        server.connect_blocking().unwrap();

        client.send(Instruction::TestInstructionOne).await.unwrap();
        client.send(Instruction::TestInstructionTwo).await.unwrap();

        server.send(Instruction::TestInstructionOne).await.unwrap();
        server.send(Instruction::TestInstructionTwo).await.unwrap();

        assert_eq!(
            client.recv().await.unwrap(),
            Instruction::TestInstructionOne
        );
        assert_eq!(
            server.recv().await.unwrap(),
            Instruction::TestInstructionOne
        );
        assert_eq!(
            client.recv().await.unwrap(),
            Instruction::TestInstructionTwo
        );
        assert_eq!(
            server.recv().await.unwrap(),
            Instruction::TestInstructionTwo
        );

        client.send(Instruction::TestInstructionOne).await.unwrap();
        client.send(Instruction::TestInstructionTwo).await.unwrap();

        server.send(Instruction::TestInstructionOne).await.unwrap();
        server.send(Instruction::TestInstructionTwo).await.unwrap();

        assert_eq!(
            client.recv().await.unwrap(),
            Instruction::TestInstructionOne
        );
        assert_eq!(
            server.recv().await.unwrap(),
            Instruction::TestInstructionOne
        );
        assert_eq!(
            client.recv().await.unwrap(),
            Instruction::TestInstructionTwo
        );
        assert_eq!(
            server.recv().await.unwrap(),
            Instruction::TestInstructionTwo
        );
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
                    c.send(Instruction::TestInstructionOne).await.unwrap();
                    c.send(Instruction::TestInstructionTwo).await.unwrap();

                    s.send(Instruction::TestInstructionOne).await.unwrap();
                    s.send(Instruction::TestInstructionTwo).await.unwrap();
                }
            });

            assert_eq!(
                client.recv().await.unwrap(),
                Instruction::TestInstructionOne
            );
            assert_eq!(
                server.recv().await.unwrap(),
                Instruction::TestInstructionOne
            );
            assert_eq!(
                client.recv().await.unwrap(),
                Instruction::TestInstructionTwo
            );
            assert_eq!(
                server.recv().await.unwrap(),
                Instruction::TestInstructionTwo
            );
        }
    }
}
