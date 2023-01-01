use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::node::NodeId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GuestToNodeMessage {
    RegisterRequest {
        address: SocketAddr,
    },
    HandshakeRequest {
        address: SocketAddr,
        node_id: NodeId,
    },
}

impl From<Message> for Result<GuestToNodeMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::GuestToNode(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GuestToNodeMessageStreamReader = BasicMessageStreamReader<GuestToNodeMessage>;
pub type GuestToNodeMessageStreamWriter = BasicMessageStreamWriter<GuestToNodeMessage>;
