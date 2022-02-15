#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("i/o error, {0}")]
    IO(#[from] std::io::Error),

    #[error("socket error, {0}")]
    Ipc(#[from] gistit_ipc::Error),

    #[error("serialize error, {0}")]
    Json(#[from] serde_json::Error),

    #[error("decode error, {0}")]
    Identity(#[from] libp2p::identity::error::DecodingError),
    #[error("decode error, {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("decode error, {0}")]
    Proto(#[from] gistit_proto::Error),

    #[error("fail to parse multiaddr: {0}")]
    Multiaddr(#[from] libp2p::multiaddr::Error),

    #[error("reference error, {0}")]
    Reference(#[from] gistit_reference::Error),

    #[error("p2p transport error, {0}")]
    Transport(#[from] libp2p::TransportError<std::io::Error>),

    #[error("dial error, {0}")]
    Dial(#[from] libp2p::swarm::DialError),

    #[error("request response codec error, {0}")]
    Codec(#[from] crate::behaviour::Response),

    #[error("parse error, {0}")]
    Parse(&'static str),
}
