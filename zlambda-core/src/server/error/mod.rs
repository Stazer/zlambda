use crate::message::MessageError;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NewServerError {
    Io(io::Error),
    Message(MessageError),
    IsOnline(ServerId),
}

impl error::Error for NewServerError {}

impl Display for NewServerError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, formatter),
            Self::Message(error) => Display::fmt(error, formatter),
            Self::IsOnline(server_id) => write!(formatter, "Server with id {} is online", server_id),
        }
    }
}

impl From<io::Error> for NewServerError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<MessageError> for NewServerError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}
