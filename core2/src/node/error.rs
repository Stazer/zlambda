use crate::error::RecoveryError;
use crate::message::{MessageError, MessageStreamReader, MessageStreamWriter};
use std::io;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NewNodeError {
    IoError(io::Error),
    Message(MessageError),
    Recovery(RecoveryError),
}

impl From<io::Error> for NewNodeError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<MessageError> for NewNodeError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}

impl From<RecoveryError> for NewNodeError {
    fn from(error: RecoveryError) -> Self {
        Self::Recovery(error)
    }
}
