use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::node::NodeId;
use crate::term::Term;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderToGuestMessage {
    RegisterOkResponse {
        id: NodeId,
        leader_id: NodeId,
        addresses: HashMap<NodeId, SocketAddr>,
        term: Term,
    },
    HandshakeErrorResponse {
        message: String,
    },
    HandshakeOkResponse {
        leader_id: NodeId,
    },
}

impl From<Message> for Result<LeaderToGuestMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::LeaderToGuest(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LeaderToGuestMessageStreamReader = BasicMessageStreamReader<LeaderToGuestMessage>;
pub type LeaderToGuestMessageStreamWriter = BasicMessageStreamWriter<LeaderToGuestMessage>;
