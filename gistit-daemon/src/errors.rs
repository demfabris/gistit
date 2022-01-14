#[derive(Debug)]
pub enum ErrorKind {
    InvalidArgs,
    FsEvent(String),
    Io(std::io::Error),
    Multiaddr(libp2p::core::multiaddr::Error),
    InvalidPeerAddress(String),
    InvalidPeerFile,
    ConnectionSwarm(libp2p::swarm::DialError),
    Internal(lib_gistit::Error),
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Self {
        Self {
            kind: ErrorKind::InvalidArgs,
        }
    }
}

impl From<libp2p::core::multiaddr::Error> for Error {
    fn from(err: libp2p::core::multiaddr::Error) -> Self {
        Self {
            kind: ErrorKind::Multiaddr(err),
        }
    }
}

impl From<libp2p::swarm::DialError> for Error {
    fn from(err: libp2p::swarm::DialError) -> Self {
        Self {
            kind: ErrorKind::ConnectionSwarm(err),
        }
    }
}

impl From<lib_gistit::Error> for Error {
    fn from(err: lib_gistit::Error) -> Self {
        Self {
            kind: ErrorKind::Internal(err),
        }
    }
}
