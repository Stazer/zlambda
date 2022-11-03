use crate::cluster::{LogEntry, LogEntryId, NodeId, TermId};
use bytes::Bytes;
use postcard::{take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeRegisterResponsePacketSuccessData {
    node_id: NodeId,
    leader_node_id: NodeId,
    term_id: TermId,
    node_socket_addresses: HashMap<NodeId, SocketAddr>,
}

impl From<NodeRegisterResponsePacketSuccessData>
    for (NodeId, NodeId, TermId, HashMap<NodeId, SocketAddr>)
{
    fn from(response: NodeRegisterResponsePacketSuccessData) -> Self {
        (
            response.node_id,
            response.leader_node_id,
            response.term_id,
            response.node_socket_addresses,
        )
    }
}

impl NodeRegisterResponsePacketSuccessData {
    pub fn new(
        node_id: NodeId,
        leader_node_id: NodeId,
        term_id: TermId,
        node_socket_addresses: HashMap<NodeId, SocketAddr>,
    ) -> Self {
        Self {
            node_id,
            leader_node_id,
            term_id,
            node_socket_addresses,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
    }

    pub fn term_id(&self) -> TermId {
        self.term_id
    }

    pub fn node_socket_addresses(&self) -> &HashMap<NodeId, SocketAddr> {
        &self.node_socket_addresses
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeRegisterResponsePacketError {
    NotALeader { leader_address: SocketAddr },
}

impl Display for NodeRegisterResponsePacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::NotALeader { .. } => write!(formatter, "Not a leader"),
        }
    }
}

impl error::Error for NodeRegisterResponsePacketError {}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LogEntryResponsePacketResult {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Packet {
    NodeRegisterRequest {
        local_address: SocketAddr,
    },
    NodeRegisterResponse {
        result: Result<NodeRegisterResponsePacketSuccessData, NodeRegisterResponsePacketError>,
    },
    NodeRegisterAcknowledgement,
    LogEntryRequest {
        log_entry: LogEntry,
    },
    LogEntrySuccessResponse {
        log_entry_id: LogEntryId,
    },
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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
pub struct PacketReader {
    buffer: Vec<u8>,
}

impl PacketReader {
    pub fn push(&mut self, bytes: Bytes) {
        self.buffer.extend(bytes);
    }

    pub fn next(&mut self) -> Result<Option<Packet>, ReadPacketError> {
        let (read, packet) = match Packet::from_vec(&self.buffer) {
            Ok((read, packet)) => (read, packet),
            Err(ReadPacketError::UnexpectedEnd) => return Ok(None),
            Err(error) => return Err(error),
        };

        self.buffer.drain(0..read);

        Ok(Some(packet))
    }
}
