use crate::common::module::{LoadModuleError};
use crate::common::message::MessageError;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NewClientError {
    Io(io::Error),
    Message(MessageError),
    LoadModule(LoadModuleError),
}

impl error::Error for NewClientError {}

impl Display for NewClientError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, formatter),
            Self::Message(error) => Display::fmt(error, formatter),
            Self::LoadModule(error) => Display::fmt(error, formatter),
        }
    }
}

impl From<io::Error> for NewClientError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<MessageError> for NewClientError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}

impl From<LoadModuleError> for NewClientError {
    fn from(error: LoadModuleError) -> Self {
        Self::LoadModule(error)
    }
}
