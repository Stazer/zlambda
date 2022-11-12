pub mod node;

////////////////////////////////////////////////////////////////////////////////////////////////////

use bytes::Bytes;
use postcard::{take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt::{self, Debug, Display, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type TermId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub enum ClusterMessage {
    RegisterRequest,
    RegisterResponse,
    AppendEntriesRequest,
    AppendEntriesResponse,
    RequestVoteRequest,
    RequestVoteResponse,
}

impl ClusterMessage {
    pub fn from_vec(bytes: &Vec<u8>) -> Result<(usize, Self), ReadClusterMessageError> {
        let (packet, remaining) = take_from_bytes::<Self>(bytes)?;
        Ok((bytes.len() - remaining.len(), packet))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, WriteClusterMessageError> {
        Ok(to_allocvec(&self)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes, WriteClusterMessageError> {
        Ok(Bytes::from(self.to_vec()?))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ReadClusterMessageError {
    UnexpectedEnd,
    PostcardError(postcard::Error),
}

impl Debug for ReadClusterMessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::PostcardError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for ReadClusterMessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::PostcardError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for ReadClusterMessageError {}

impl From<postcard::Error> for ReadClusterMessageError {
    fn from(error: postcard::Error) -> Self {
        match error {
            postcard::Error::DeserializeUnexpectedEnd => Self::UnexpectedEnd,
            e => Self::PostcardError(e),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum WriteClusterMessageError {
    PostcardError(postcard::Error),
}

impl Debug for WriteClusterMessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for WriteClusterMessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for WriteClusterMessageError {}

impl From<postcard::Error> for WriteClusterMessageError {
    fn from(error: postcard::Error) -> Self {
        Self::PostcardError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
pub struct ClusterMessageReader {
    buffer: Vec<u8>,
}

impl ClusterMessageReader {
    pub fn push(&mut self, bytes: Bytes) {
        self.buffer.extend(bytes);
    }

    pub fn next(&mut self) -> Result<Option<ClusterMessage>, ReadClusterMessageError> {
        let (read, message) = match ClusterMessage::from_vec(&self.buffer) {
            Ok((read, message)) => (read, message),
            Err(ReadClusterMessageError::UnexpectedEnd) => return Ok(None),
            Err(error) => return Err(error),
        };

        self.buffer.drain(0..read);

        Ok(Some(message))
    }
}
