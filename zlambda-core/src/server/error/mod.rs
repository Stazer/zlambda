use crate::common::message::MessageError;
use crate::common::module::LoadModuleError;
use crate::server::ServerId;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NewServerError {
    Io(io::Error),
    Message(MessageError),
    IsOnline(ServerId),
    Unknown(ServerId),
    LoadModule(LoadModuleError),
}

impl error::Error for NewServerError {}

impl Display for NewServerError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, formatter),
            Self::Message(error) => Display::fmt(error, formatter),
            Self::IsOnline(server_id) => {
                write!(formatter, "Server with id {server_id} is online")
            }
            Self::Unknown(server_id) => {
                write!(formatter, "Server id {server_id} is unknown")
            }
            Self::LoadModule(error) => Display::fmt(error, formatter),
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

impl From<LoadModuleError> for NewServerError {
    fn from(error: LoadModuleError) -> Self {
        Self::LoadModule(error)
    }
}
