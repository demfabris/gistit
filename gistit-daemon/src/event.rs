use std::str;

use libp2p::identify::{IdentifyEvent, IdentifyInfo};
use libp2p::kad::record::Key;
use libp2p::kad::{GetProvidersError, GetProvidersOk, KademliaEvent, QueryResult};
use libp2p::multiaddr::Protocol;
use libp2p::request_response::{RequestResponseEvent, RequestResponseMessage};

use gistit_proto::Instruction;
use log::{debug, error, info};

use crate::behaviour::{Request, Response};
use crate::node::Node;
use crate::Result;

pub async fn handle_request_response(
    node: &mut Node,
    event: RequestResponseEvent<Request, Response>,
) -> Result<()> {
    match event {
        RequestResponseEvent::Message { message, .. } => match message {
            RequestResponseMessage::Request {
                request, channel, ..
            } => {
                let key = Key::new(&request.0);
                info!("Request response 'Message::Request' for {:?}", key);
                let file = node
                    .to_provide
                    .get(&key)
                    .expect("to be providing {key}")
                    .clone();

                node.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, Response(file))?;
            }
            RequestResponseMessage::Response {
                request_id,
                response,
            } => {
                info!("Request response 'Message::Response'");
                let gistit = response.0;
                let key = Key::new(&gistit.hash.as_bytes());

                if node.pending_receive_file.remove(&key) {
                    node.bridge.connect_blocking()?;
                    node.bridge
                        .send(Instruction::respond_fetch(Some(gistit)))
                        .await?;
                }
                node.pending_request_file.remove(&request_id);
            }
        },
        RequestResponseEvent::OutboundFailure {
            request_id, error, ..
        } => {
            error!("Request response outbound failure {:?}", error);
            node.pending_request_file.remove(&request_id);
            node.bridge.connect_blocking()?;
            node.bridge.send(Instruction::respond_fetch(None)).await?;
        }
        RequestResponseEvent::InboundFailure { error, .. } => {
            error!("Request response inbound failure {:?}", error);
        }
        RequestResponseEvent::ResponseSent { .. } => (),
    }
    Ok(())
}

pub async fn handle_kademlia(node: &mut Node, event: KademliaEvent) -> Result<()> {
    match event {
        KademliaEvent::OutboundQueryCompleted {
            id,
            result: QueryResult::StartProviding(maybe_provided),
            ..
        } => {
            node.pending_start_providing.remove(&id);
            node.bridge.connect_blocking()?;

            match maybe_provided {
                Ok(provider) => {
                    info!("Kademlia start providing: {:?}", provider);
                    let hash = str::from_utf8(&provider.key.to_vec())
                        .expect("hash format to be valid utf8")
                        .to_owned();
                    node.bridge
                        .send(Instruction::respond_provide(Some(hash)))
                        .await?;
                }
                Err(provider) => {
                    error!("Kademlia start providing failed: {:?}", provider);
                    node.to_provide.remove(provider.key());
                    node.bridge.send(Instruction::respond_provide(None)).await?;
                }
            }
            Ok(())
        }
        KademliaEvent::OutboundQueryCompleted {
            id,
            result: QueryResult::GetProviders(maybe_providers),
            ..
        } => {
            info!("Kademlia get providers: {:?}", maybe_providers);
            node.pending_get_providers.remove(&id);

            match maybe_providers {
                Ok(GetProvidersOk { key, providers, .. }) => {
                    node.to_request.push((key, providers));
                }
                Err(GetProvidersError::Timeout { key, .. }) => {
                    error!("No providers for {:?}", key);
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn handle_identify(node: &mut Node, event: IdentifyEvent) -> Result<()> {
    if let IdentifyEvent::Received {
        peer_id,
        info:
            IdentifyInfo {
                listen_addrs,
                protocols,
                ..
            },
    } = event
    {
        debug!("Identify: {:?}, protocols: {:?}", listen_addrs, protocols);
        if protocols.iter().any(|p| p.as_bytes() == KADEMLIA_PROTO) {
            for addr in &listen_addrs {
                node.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
            }
        }

        if protocols.iter().any(|p| p.as_bytes() == RELAY_HOP_PROTO) {
            for addr in listen_addrs {
                // Don't attempt to relay over the relay
                if addr.iter().any(|t| matches!(t, Protocol::P2pCircuit)) {
                    continue;
                }

                let peer = addr
                    .with(Protocol::P2p(peer_id.into()))
                    .with(Protocol::P2pCircuit);

                if node.relays.contains(&peer) {
                    continue;
                }

                info!("Listening on relay {:?}", peer);
                node.relays.insert(peer.clone());
                node.swarm.listen_on(peer)?;
            }
        }
    }
    Ok(())
}

const KADEMLIA_PROTO: &[u8] = b"/ipfs/kad/1.0.0";
const RELAY_HOP_PROTO: &[u8] = b"/libp2p/circuit/relay/0.2.0/hop";
// const RELAY_STOP_PROTO: &[u8] = b"/libp2p/circuit/relay/0.2.0/stop";
