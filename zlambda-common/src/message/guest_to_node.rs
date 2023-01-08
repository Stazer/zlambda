use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::node::NodeId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuestToNodeRegisterRequestMessage {
    address: SocketAddr,
}

impl From<GuestToNodeRegisterRequestMessage> for (SocketAddr,) {
    fn from(message: GuestToNodeRegisterRequestMessage) -> Self {
        (message.address,)
    }
}

impl GuestToNodeRegisterRequestMessage {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuestToNodeHandshakeRequestMessage {
    address: SocketAddr,
    node_id: NodeId,
}

impl From<GuestToNodeHandshakeRequestMessage> for (SocketAddr, NodeId) {
    fn from(message: GuestToNodeHandshakeRequestMessage) -> Self {
        (message.address, message.node_id)
    }
}

impl GuestToNodeHandshakeRequestMessage {
    pub fn new(address: SocketAddr, node_id: NodeId) -> Self {
        Self { address, node_id }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GuestToNodeMessage {
    RegisterRequest(GuestToNodeRegisterRequestMessage),
    HandshakeRequest(GuestToNodeHandshakeRequestMessage),
}

impl From<Message> for Result<GuestToNodeMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::GuestToNode(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GuestToNodeMessageStreamReader = BasicMessageStreamReader<GuestToNodeMessage>;
pub type GuestToNodeMessageStreamWriter = BasicMessageStreamWriter<GuestToNodeMessage>;
