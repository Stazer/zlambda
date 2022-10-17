use crate::cluster::{NodeId, TermId, LogEntry, LogEntryId};
use bytes::Bytes;
use postcard::{take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeRegisterResponsePacketSuccessNodeData {
    node_id: NodeId,
    socket_address: SocketAddr,
}

impl From<NodeRegisterResponsePacketSuccessNodeData> for (NodeId, SocketAddr) {
    fn from(response: NodeRegisterResponsePacketSuccessNodeData) -> Self {
        (response.node_id, response.socket_address)
    }
}

impl NodeRegisterResponsePacketSuccessNodeData {
    pub fn new(node_id: NodeId, socket_address: SocketAddr) -> Self {
        Self {
            node_id,
            socket_address,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeRegisterResponsePacketSuccessData {
    node_id: NodeId,
    term_id: TermId,
    nodes: Vec<NodeRegisterResponsePacketSuccessNodeData>,
    leader_node_id: NodeId,
}

impl From<NodeRegisterResponsePacketSuccessData>
    for (
        NodeId,
        TermId,
        Vec<NodeRegisterResponsePacketSuccessNodeData>,
        NodeId,
    )
{
    fn from(response: NodeRegisterResponsePacketSuccessData) -> Self {
        (
            response.node_id,
            response.term_id,
            response.nodes,
            response.leader_node_id,
        )
    }
}

impl NodeRegisterResponsePacketSuccessData {
    pub fn new(
        node_id: NodeId,
        term_id: TermId,
        nodes: Vec<NodeRegisterResponsePacketSuccessNodeData>,
        leader_node_id: NodeId,
    ) -> Self {
        Self {
            node_id,
            term_id,
            nodes,
            leader_node_id,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn term_id(&self) -> TermId {
        self.term_id
    }

    pub fn nodes(&self) -> &Vec<NodeRegisterResponsePacketSuccessNodeData> {
        &self.nodes
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeUpdateNodeData {
    node_id: NodeId,
    socket_address: SocketAddr
}

impl From<NodeUpdateNodeData> for (NodeId, SocketAddr) {
    fn from(data: NodeUpdateNodeData) -> Self {
        (data.node_id, data.socket_address)
    }
}

impl NodeUpdateNodeData {
    pub fn new(node_id: NodeId, socket_address: SocketAddr) -> Self {
        Self {
            node_id,
            socket_address,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Packet {
    Ping,
    Pong,
    NodeRegisterRequest {
        local_address: SocketAddr,
    },
    NodeRegisterResponse {
        result: Result<NodeRegisterResponsePacketSuccessData, NodeRegisterResponsePacketError>,
    },
    NodeRegisterAcknowledgement,
    NodeUpdate {
        nodes: Vec<NodeUpdateNodeData>,
    },
    LogEntryRequest {
        log_entry: LogEntry,
    },
    LogEntryResponse {
        log_entry_id: LogEntryId,
    },
    LogEntryAcknowledgement {
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
