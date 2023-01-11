use crate::node::member::NodeMemberReference;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use zlambda_common::error::FollowerRegistrationError;
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodePingMessage {
    sender: oneshot::Sender<()>,
}

impl From<NodePingMessage> for (oneshot::Sender<()>,) {
    fn from(message: NodePingMessage) -> Self {
        (message.sender,)
    }
}

impl From<NodePingMessage> for NodeMessage {
    fn from(message: NodePingMessage) -> Self {
        Self::Ping(message)
    }
}

impl NodePingMessage {
    pub fn new(sender: oneshot::Sender<()>) -> Self {
        Self { sender }
    }

    pub fn sender(&self) -> &oneshot::Sender<()> {
        &self.sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeSocketAcceptMessage {
    socket_address: SocketAddr,
    socket: TcpStream,
}

impl From<NodeSocketAcceptMessage> for (SocketAddr, TcpStream) {
    fn from(message: NodeSocketAcceptMessage) -> Self {
        (message.socket_address, message.socket)
    }
}

impl From<NodeSocketAcceptMessage> for NodeMessage {
    fn from(message: NodeSocketAcceptMessage) -> Self {
        Self::SocketAccept(message)
    }
}

impl NodeSocketAcceptMessage {
    pub fn new(socket_address: SocketAddr, socket: TcpStream) -> Self {
        Self {
            socket_address,
            socket,
        }
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn socket(&self) -> &TcpStream {
        &self.socket
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollowerRegistrationMessage {
    address: SocketAddr,
    sender: oneshot::Sender<Result<NodeMemberReference, FollowerRegistrationError>>,
}

impl From<NodeFollowerRegistrationMessage>
    for (
        SocketAddr,
        oneshot::Sender<Result<NodeMemberReference, FollowerRegistrationError>>,
    )
{
    fn from(message: NodeFollowerRegistrationMessage) -> Self {
        (message.address, message.sender)
    }
}

impl From<NodeFollowerRegistrationMessage> for NodeMessage {
    fn from(message: NodeFollowerRegistrationMessage) -> Self {
        Self::FollowerRegistration(message)
    }
}

impl NodeFollowerRegistrationMessage {
    pub fn new(
        address: SocketAddr,
        sender: oneshot::Sender<Result<NodeMemberReference, FollowerRegistrationError>>,
    ) -> Self {
        Self { address, sender }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn sender(
        &self,
    ) -> &oneshot::Sender<Result<NodeMemberReference, FollowerRegistrationError>> {
        &self.sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollowerHandshakeMessage {
    node_id: NodeId,
}

impl From<NodeFollowerHandshakeMessage> for (NodeId,) {
    fn from(message: NodeFollowerHandshakeMessage) -> Self {
        (message.node_id,)
    }
}

impl From<NodeFollowerHandshakeMessage> for NodeMessage {
    fn from(message: NodeFollowerHandshakeMessage) -> Self {
        Self::FollowerHandshake(message)
    }
}

impl NodeFollowerHandshakeMessage {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollowerRecoveryMessage {
    node_id: NodeId,
}

impl From<NodeFollowerRecoveryMessage> for (NodeId,) {
    fn from(message: NodeFollowerRecoveryMessage) -> Self {
        (message.node_id,)
    }
}

impl From<NodeFollowerRecoveryMessage> for NodeMessage {
    fn from(message: NodeFollowerRecoveryMessage) -> Self {
        Self::FollowerRecovery(message)
    }
}

impl NodeFollowerRecoveryMessage {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeMessage {
    Ping(NodePingMessage),
    SocketAccept(NodeSocketAcceptMessage),
    FollowerRegistration(NodeFollowerRegistrationMessage),
    FollowerHandshake(NodeFollowerHandshakeMessage),
    FollowerRecovery(NodeFollowerRecoveryMessage),
    //FollowerHandshake
    //FollowerRecovery
}
