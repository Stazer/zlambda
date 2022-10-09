use crate::cluster::NodeId;
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
    nodes: Vec<NodeRegisterResponsePacketSuccessNodeData>,
}

impl From<NodeRegisterResponsePacketSuccessData>
    for (NodeId, Vec<NodeRegisterResponsePacketSuccessNodeData>)
{
    fn from(response: NodeRegisterResponsePacketSuccessData) -> Self {
        (response.node_id, response.nodes)
    }
}

impl NodeRegisterResponsePacketSuccessData {
    pub fn new(node_id: NodeId, nodes: Vec<NodeRegisterResponsePacketSuccessNodeData>) -> Self {
        Self { node_id, nodes }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn nodes(&self) -> &Vec<NodeRegisterResponsePacketSuccessNodeData> {
        &self.nodes
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeRegisterResponsePacketError {
    NotRegistered,
    NotALeader { leader_address: SocketAddr },
}

impl Display for NodeRegisterResponsePacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::NotRegistered => write!(formatter, "Not registered"),
            Self::NotALeader { .. } => write!(formatter, "Not a leader"),
        }
    }
}

impl error::Error for NodeRegisterResponsePacketError {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Packet {
    NodeRegisterRequest,
    NodeRegisterResponse {
        result: Result<NodeId, NodeRegisterResponsePacketError>,
    },
    NodePing,
    NodePong,
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
