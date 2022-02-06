//! Handle behaviour events

use libp2p::kad::record::Key;
use libp2p::kad::{GetProvidersError, GetProvidersOk, KademliaEvent, QueryId, QueryResult};
use libp2p::request_response::{RequestId, RequestResponseEvent, RequestResponseMessage};

use gistit_ipc::{Instruction, ServerResponse};
use log::{debug, error};

use crate::behaviour::{Request, Response};
use crate::network::Node;
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
                debug!("Request response 'Message::Request' for {:?}", key);
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
                debug!("Request response 'Message::Response'");
                node.pending_request_file.remove(&request_id);
                node.bridge.connect_blocking()?;
                node.bridge
                    .send(Instruction::Response(ServerResponse::File(response.0)))
                    .await?;
            }
        },
        RequestResponseEvent::OutboundFailure {
            request_id, error, ..
        } => {
            error!("{:?}", error);
            node.pending_request_file.remove(&request_id);
        }
        RequestResponseEvent::InboundFailure { error, .. } => {
            error!("{:?}", error);
        }
        RequestResponseEvent::ResponseSent { .. } => (),
    }
    Ok(())
}

pub fn handle_kademlia(node: &mut Node, event: KademliaEvent) {
    match event {
        KademliaEvent::OutboundQueryCompleted {
            id,
            result: QueryResult::StartProviding(maybe_provided),
            ..
        } => {
            node.pending_start_providing.remove(&id);
            debug!(
                "Kademlia start providing: {:?}",
                maybe_provided.as_ref().unwrap()
            );

            // Failed to provide, remove from providing map
            if let Err(provider) = maybe_provided {
                node.to_provide.remove(provider.key());
            }
        }
        KademliaEvent::OutboundQueryCompleted {
            id,
            result: QueryResult::GetProviders(maybe_providers),
            ..
        } => {
            debug!("Kademlia get providers: {:?}", maybe_providers);
            node.pending_get_providers.remove(&id);

            match maybe_providers {
                Ok(GetProvidersOk { key, providers, .. }) => {
                    node.to_request.push((key, providers));
                }
                Err(GetProvidersError::Timeout { key, .. }) => {
                    error!("No providers for {:?}", key);
                }
            }
        }
        _ => (),
    }
}
