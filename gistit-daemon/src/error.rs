#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("fail to parse multiaddr: {0}")]
    Multiaddr(#[from] libp2p::multiaddr::Error),

    #[error("i/o error, {0}")]
    IO(#[from] std::io::Error),

    #[error("socket error, {0}")]
    Ipc(#[from] gistit_ipc::Error),

    #[error("p2p transport error, {0}")]
    Transport(#[from] libp2p::TransportError<std::io::Error>),

    #[error("dial error, {0}")]
    Dial(#[from] libp2p::swarm::DialError),
}
