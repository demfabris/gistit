//! Inter process comunication module
#![allow(clippy::missing_errors_doc)]

use std::fs::{metadata, remove_file};
use std::marker::PhantomData;
use std::net::Ipv4Addr;
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

const NAMED_SOCKET_0: &str = "gistit-0";
const NAMED_SOCKET_1: &str = "gistit-1";
const READBUF_SIZE: usize = 60_000;
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

    Ok(Bridge {
        sock_0: UnixDatagram::bind(sockpath_0)?,
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

    Ok(Bridge {
        sock_0: UnixDatagram::unbound()?,
        sock_1: UnixDatagram::bind(sockpath_1)?,
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
    Ok(())
}

impl Bridge<Server> {
    pub fn alive(&self) -> bool {
        __alive(&self.base, &self.sock_1, NAMED_SOCKET_1)
    }

    pub fn connect_blocking(&mut self) -> Result<()> {
        __connect_blocking(&self.base, &self.sock_1, NAMED_SOCKET_1)
    }

    pub fn send(&self, instruction: Instruction) -> Result<()> {
        let encoded = serialize(&instruction).unwrap();
        self.sock_1.send(&encoded)?;
        Ok(())
    }

    pub fn recv(&self) -> Result<Instruction> {
        let mut buf = vec![0u8; READBUF_SIZE];
        self.sock_0.recv(&mut buf)?;
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

    pub fn send(&self, instruction: Instruction) -> Result<()> {
        let encoded = serialize(&instruction).unwrap();
        self.sock_0.send(&encoded)?;
        Ok(())
    }

    pub fn recv(&self) -> Result<Instruction> {
        let mut buf = vec![0u8; READBUF_SIZE];
        self.sock_1.recv(&mut buf)?;
        let target = deserialize(&buf).unwrap();
        Ok(target)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Instruction {
    Listen { host: Ipv4Addr, port: u16 },
    Dial { peer_id: String },
    Provide { hash: String, data: Vec<u8> },
    Get { hash: String },
    Status,
    Shutdown,
    // Daemon responses
    Response(ServerResponse),
}

#[derive(Serialize, Deserialize, Debug)]
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
