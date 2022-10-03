use crate::cluster::NodeId;
use bytes::Bytes;
use postcard::{take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type CommandType = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConsensusPacket {
    //BeginRequest(ConsensusBeginRequest<CommandType>),
    //BeginResponse(ConsensusBeginResponse<CommandType>),
    //CommitRequest(ConsensusCommitRequest<CommandType>),
    //CommitResponse(ConsensusCommitResponse<CommandType>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Packet {
    NodeHandshakeRequest,
    NodeHandshakeResponse,
    ClientHandshakeRequest,
    ClientHandshakeResponse,

    NodeRegisterRequest,
    NodeRegisterResponse { result: Result<NodeId, ()> },

    Consensus(ConsensusPacket),
}

impl Packet {
    pub fn from_vec(bytes: &Vec<u8>) -> Result<(usize, Self), ReadPacketError> {
        let (packet, remaining) = take_from_bytes::<Self>(bytes)?;
        Ok((bytes.len() - remaining.len(), packet))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, WritePacketError> {
        Ok(to_allocvec(&self)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes, WritePacketError> {
        Ok(Bytes::from(self.to_vec()?))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ReadPacketError {
    UnexpectedEnd,
    PostcardError(postcard::Error),
}

impl Debug for ReadPacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::PostcardError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for ReadPacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::PostcardError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for ReadPacketError {}

impl From<postcard::Error> for ReadPacketError {
    fn from(error: postcard::Error) -> Self {
        match error {
            postcard::Error::DeserializeUnexpectedEnd => Self::UnexpectedEnd,
            e => Self::PostcardError(e),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum WritePacketError {
    PostcardError(postcard::Error),
}

impl Debug for WritePacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for WritePacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for WritePacketError {}

impl From<postcard::Error> for WritePacketError {
    fn from(error: postcard::Error) -> Self {
        Self::PostcardError(error)
    }
}
