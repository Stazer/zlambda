use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FollowerToGuestRegisterNotALeaderResponseMessage {
    leader_address: SocketAddr,
}

impl From<FollowerToGuestRegisterNotALeaderResponseMessage> for (SocketAddr,) {
    fn from(message: FollowerToGuestRegisterNotALeaderResponseMessage) -> Self {
        (message.leader_address,)
    }
}

impl FollowerToGuestRegisterNotALeaderResponseMessage {
    pub fn new(leader_address: SocketAddr) -> Self {
        Self { leader_address }
    }

    pub fn leader_address(&self) -> &SocketAddr {
        &self.leader_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FollowerToGuestRecoveryNotALeaderResponseMessage {
    leader_address: SocketAddr,
}

impl From<FollowerToGuestRecoveryNotALeaderResponseMessage> for (SocketAddr,) {
    fn from(message: FollowerToGuestRecoveryNotALeaderResponseMessage) -> Self {
        (message.leader_address,)
    }
}

impl FollowerToGuestRecoveryNotALeaderResponseMessage {
    pub fn new(leader_address: SocketAddr) -> Self {
        Self { leader_address }
    }

    pub fn leader_address(&self) -> &SocketAddr {
        &self.leader_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToGuestMessage {
    RegisterNotALeaderResponse(FollowerToGuestRegisterNotALeaderResponseMessage),
    RecoveryNotALeaderResponse(FollowerToGuestRecoveryNotALeaderResponseMessage),
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

pub type FollowerToGuestMessageStreamReader = BasicMessageStreamReader<FollowerToGuestMessage>;
pub type FollowerToGuestMessageStreamWriter = BasicMessageStreamWriter<FollowerToGuestMessage>;
