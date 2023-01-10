use std::error::Error;
use std::io;
use zlambda_common::message::MessageError;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum FollowerClientResult {
    Continue,
    Stop,
    ConnectionClosed,
    Error(Box<dyn Error + Send>),
}

impl From<Box<dyn Error + Send>> for FollowerClientResult {
    fn from(error: Box<dyn Error + Send>) -> Self {
        Self::Error(error)
    }
}

impl From<MessageError> for FollowerClientResult {
    fn from(error: MessageError) -> Self {
        Self::Error(Box::new(error))
    }
}

impl From<io::Error> for FollowerClientResult {
    fn from(error: io::Error) -> Self {
        if matches!(error.kind(), io::ErrorKind::ConnectionReset) {
            return Self::ConnectionClosed;
        }

        Self::Error(Box::new(error))
    }
}
