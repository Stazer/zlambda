use std::io;
use zlambda_common::message::MessageError;
use zlambda_common::error::RecoveryError;

////////////////////////////////////////////////////////////////////////////////////////////////////

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
