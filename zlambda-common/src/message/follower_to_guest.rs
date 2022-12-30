use crate::message::{Message, MessageError, BasicMessageStreamReader, BasicMessageStreamWriter};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToGuestMessage {
    RegisterNotALeaderResponse { leader_address: SocketAddr },
    HandshakeNotALeaderResponse { leader_address: SocketAddr },
}

impl From<Message> for Result<FollowerToGuestMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::FollowerToGuest(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type FollowerToGuestMessageStreamReader =
    BasicMessageStreamReader<FollowerToGuestMessage>;
pub type FollowerToGuestMessageStreamWriter =
    BasicMessageStreamWriter<FollowerToGuestMessage>;
