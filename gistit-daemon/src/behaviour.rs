use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::core::ProtocolName;
use libp2p::futures::{AsyncRead, AsyncWrite, AsyncWriteExt};
use libp2p::NetworkBehaviour;

use libp2p::autonat::{Behaviour as Autonat, Event as AutonatEvent};
use libp2p::identify::{Identify, IdentifyEvent};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{Kademlia, KademliaEvent};
use libp2p::relay::v2::relay::{Event as RelayEvent, Relay};
use libp2p::request_response::{RequestResponse, RequestResponseCodec, RequestResponseEvent};

use async_trait::async_trait;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    pub request_response: RequestResponse<ExchangeCodec>,
    pub kademlia: Kademlia<MemoryStore>,
    pub identify: Identify,
    pub relay: Relay,
    pub autonat: Autonat,
}

#[derive(Debug)]
pub enum Event {
    RequestResponse(RequestResponseEvent<Request, Response>),
    Kademlia(KademliaEvent),
    Identify(IdentifyEvent),
    Relay(RelayEvent),
    Autonat(AutonatEvent),
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

#[derive(Debug, Clone)]
pub struct ExchangeProtocol;

impl ProtocolName for ExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/gistit/1"
    }
}

#[derive(Clone)]
pub struct ExchangeCodec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request(Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response(Vec<u8>);

const MAX_FILE_SIZE: usize = 50_000;

#[async_trait]
impl RequestResponseCodec for ExchangeCodec {
    type Protocol = ExchangeProtocol;
    type Request = Request;
    type Response = Response;

    async fn read_request<T: Send + Unpin + AsyncRead>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> tokio::io::Result<Self::Request> {
        let bytes = read_length_prefixed(io, MAX_FILE_SIZE).await?;

        if bytes.is_empty() {
            Err(tokio::io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(Request(bytes))
        }
    }

    async fn read_response<T: Send + Unpin + AsyncRead>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> tokio::io::Result<Self::Response> {
        let bytes = read_length_prefixed(io, MAX_FILE_SIZE).await?;

        if bytes.is_empty() {
            Err(tokio::io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(Response(bytes))
        }
    }

    async fn write_request<T: Send + Unpin + AsyncWrite>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        Request(data): Self::Request,
    ) -> tokio::io::Result<()> {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        Response(data): Self::Response,
    ) -> tokio::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
