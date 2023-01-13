use crate::error::RecoveryError;
use crate::message::{MessageError, MessageStreamReader, MessageStreamWriter};
use std::io;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum CreateNodeError {
    IoError(io::Error),
    Message(MessageError),
    Recovery(RecoveryError),
}

impl From<io::Error> for CreateNodeError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<MessageError> for CreateNodeError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}

impl From<RecoveryError> for CreateNodeError {
    fn from(error: RecoveryError) -> Self {
        Self::Recovery(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollowerRegistrationAttemptNotALeaderError {
    leader_address: SocketAddr,
}

impl From<NodeFollowerRegistrationAttemptNotALeaderError> for (SocketAddr,) {
    fn from(error: NodeFollowerRegistrationAttemptNotALeaderError) -> Self {
        (error.leader_address,)
    }
}

impl From<NodeFollowerRegistrationAttemptNotALeaderError> for NodeFollowerRegistrationAttemptError {
    fn from(error: NodeFollowerRegistrationAttemptNotALeaderError) -> Self {
        Self::NotALeader(error)
    }
}

impl NodeFollowerRegistrationAttemptNotALeaderError {
    pub fn new(leader_address: SocketAddr) -> Self {
        Self { leader_address }
    }

    pub fn leader_address(&self) -> &SocketAddr {
        &self.leader_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeFollowerRegistrationAttemptError {
    NotALeader(NodeFollowerRegistrationAttemptNotALeaderError),
}
