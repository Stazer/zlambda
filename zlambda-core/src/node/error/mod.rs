use crate::message::{MessageError};
use std::io;
use std::net::SocketAddr;
use std::fmt::{self, Display, Formatter};
use std::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NewNodeError {
    Io(io::Error),
    Message(MessageError),
}

impl error::Error for NewNodeError {}

impl Display for NewNodeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, formatter),
            Self::Message(error) => Display::fmt(error, formatter),
        }
    }
}

impl From<io::Error> for NewNodeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<MessageError> for NewNodeError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}
