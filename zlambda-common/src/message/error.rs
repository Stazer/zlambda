use crate::message::Message;
use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum MessageError {
    UnexpectedEnd,
    UnexpectedMessage(Message),
    ExpectedMessage,
    PostcardError(postcard::Error),
    IoError(io::Error),
}

impl Debug for MessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::UnexpectedMessage(_) => write!(formatter, "Unexpected message"),
            Self::ExpectedMessage => write!(formatter, "Expected message"),
            Self::PostcardError(error) => Debug::fmt(error, formatter),
            Self::IoError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for MessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::UnexpectedMessage(_) => write!(formatter, "Unexpected message"),
            Self::ExpectedMessage => write!(formatter, "Expected message"),
            Self::PostcardError(error) => Display::fmt(error, formatter),
            Self::IoError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for MessageError {}

impl From<postcard::Error> for MessageError {
    fn from(error: postcard::Error) -> Self {
        match error {
            postcard::Error::DeserializeUnexpectedEnd => Self::UnexpectedEnd,
            e => Self::PostcardError(e),
        }
    }
}

impl From<io::Error> for MessageError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}
