use std::io;
use std::iter::once;
use std::str::{self, FromStr};
use std::time::Duration;

use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::core::ProtocolName;
use libp2p::futures::{AsyncRead, AsyncWrite, AsyncWriteExt};
use libp2p::{autonat, Multiaddr, NetworkBehaviour};

use libp2p::autonat::{Behaviour as Autonat, Event as AutonatEvent};
use libp2p::core::PeerId;
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{Kademlia, KademliaConfig, KademliaEvent};
use libp2p::ping::{Behaviour as PingBehaviour, Config as PingConfig, Event as PingEvent, Ping};
use libp2p::relay::v2::client::{self, Client, Event as ClientEvent};
use libp2p::relay::v2::relay::{self, Event as RelayEvent, Relay};
use libp2p::request_response::{
    ProtocolSupport, RequestResponse, RequestResponseCodec, RequestResponseConfig,
    RequestResponseEvent,
};

use async_trait::async_trait;

use gistit_proto::prost::Message;
use gistit_proto::Gistit;

use crate::config::Config;
use crate::Result;

pub const BOOTNODES: [&str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

pub const BOOTADDR: &str = "/dnsaddr/bootstrap.libp2p.io";

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    pub request_response: RequestResponse<ExchangeCodec>,
    pub kademlia: Kademlia<MemoryStore>,
    pub identify: Identify,
    pub relay: Relay,
    pub autonat: Autonat,
    pub ping: Ping,
    pub client: Client,
}

impl Behaviour {
    pub fn new_behaviour_and_transport(
        config: &Config,
    ) -> Result<(Self, client::transport::ClientTransport)> {
        let request_response = RequestResponse::new(
            ExchangeCodec,
            once((ExchangeProtocol, ProtocolSupport::Full)),
            RequestResponseConfig::default(),
        );

        let kademlia = {
            let mut cfg = KademliaConfig::default();
            cfg.set_query_timeout(Duration::from_secs(5 * 60));
            let store = MemoryStore::new(config.peer_id);
            let mut behaviour = Kademlia::with_config(config.peer_id, store, cfg);

            let bootaddr = Multiaddr::from_str(BOOTADDR)?;
            if config.bootstrap {
                for peer in BOOTNODES {
                    behaviour.add_address(
                        &PeerId::from_str(peer).expect("peer id to be valid"),
                        bootaddr.clone(),
                    );
                }

                behaviour.bootstrap().expect("to bootstrap");
            }
            behaviour
        };

        let identify = Identify::new(IdentifyConfig::new(
            "/ipfs/0.1.0".into(),
            config.keypair.public(),
        ));

        let relay = relay::Relay::new(
            PeerId::from(config.keypair.public()),
            relay::Config::default(),
        );

        let (client_transport, client) =
            client::Client::new_transport_and_behaviour(config.peer_id);

        let autonat = {
            let mut behaviour = autonat::Behaviour::new(
                PeerId::from(config.keypair.public()),
                autonat::Config::default(),
            );
            if config.bootstrap {
                for peer in BOOTNODES {
                    let bootaddr = Multiaddr::from_str(BOOTADDR)?;
                    behaviour.add_server(
                        PeerId::from_str(peer).expect("peer id to be valid"),
                        Some(bootaddr),
                    );
                }
            }

            behaviour
        };

        let ping = PingBehaviour::new(PingConfig::new().with_keep_alive(true));

        Ok((
            Self {
                request_response,
                kademlia,
                identify,
                relay,
                autonat,
                ping,
                client,
            },
            client_transport,
        ))
    }
}

#[derive(Debug)]
pub enum Event {
    RequestResponse(RequestResponseEvent<Request, Response>),
    Kademlia(KademliaEvent),
    Identify(IdentifyEvent),
    Relay(RelayEvent),
    Autonat(AutonatEvent),
    Ping(PingEvent),
    Client(ClientEvent),
}

impl From<RequestResponseEvent<Request, Response>> for Event {
    fn from(event: RequestResponseEvent<Request, Response>) -> Self {
        Self::RequestResponse(event)
    }
}

impl From<KademliaEvent> for Event {
    fn from(event: KademliaEvent) -> Self {
        Self::Kademlia(event)
    }
}

impl From<IdentifyEvent> for Event {
    fn from(event: IdentifyEvent) -> Self {
        Self::Identify(event)
    }
}

impl From<RelayEvent> for Event {
    fn from(event: RelayEvent) -> Self {
        Self::Relay(event)
    }
}

impl From<AutonatEvent> for Event {
    fn from(event: AutonatEvent) -> Self {
        Self::Autonat(event)
    }
}

impl From<PingEvent> for Event {
    fn from(event: PingEvent) -> Self {
        Self::Ping(event)
    }
}

impl From<ClientEvent> for Event {
    fn from(event: ClientEvent) -> Self {
        Self::Client(event)
    }
}

#[derive(Debug, Clone)]
pub struct ExchangeProtocol;

impl ProtocolName for ExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/gistit/1"
    }
}

#[derive(Clone)]
pub struct ExchangeCodec;

#[derive(Debug, Clone, PartialEq)]
pub struct Request(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq)]
pub struct Response(pub Gistit);

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for Response {
    fn description(&self) -> &str {
        "failed to respond"
    }
}

const MAX_FILE_SIZE: usize = 50_000;
const HASH_SIZE: usize = 32;

#[async_trait]
impl RequestResponseCodec for ExchangeCodec {
    type Protocol = ExchangeProtocol;
    type Request = Request;
    type Response = Response;

    async fn read_request<T: Send + Unpin + AsyncRead>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request> {
        let hash = read_length_prefixed(io, HASH_SIZE).await?;

        if hash.is_empty() {
            Err(io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(Request(hash))
        }
    }

    async fn read_response<T: Send + Unpin + AsyncRead>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response> {
        let bytes = read_length_prefixed(io, MAX_FILE_SIZE).await?;
        let gistit = Gistit::decode(&*bytes).map_err(|_| io::ErrorKind::InvalidInput)?;

        if bytes.is_empty() {
            Err(io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(Response(gistit))
        }
    }

    async fn write_request<T: Send + Unpin + AsyncWrite>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        Request(gistit): Self::Request,
    ) -> io::Result<()> {
        let mut buf = vec![0u8; MAX_FILE_SIZE];
        gistit
            .encode(&mut buf)
            .map_err(|_| io::ErrorKind::InvalidInput)?;

        write_length_prefixed(io, buf).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        Response(gistit): Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let mut buf = vec![0u8; MAX_FILE_SIZE];
        gistit
            .encode(&mut buf)
            .map_err(|_| io::ErrorKind::InvalidInput)?;
        write_length_prefixed(io, buf).await?;
        io.close().await?;

        Ok(())
    }
}
