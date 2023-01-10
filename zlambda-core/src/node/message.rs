use tokio::sync::oneshot;
use tokio::net::TcpStream;
use std::net::SocketAddr;

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
    socket: TcpStream
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
pub enum NodeMessage {
    Ping(NodePingMessage),
    SocketAccept(NodeSocketAcceptMessage),
    //ClientRegistration,
    //FollowerRegistration
    //FollowerHandshake
    //FollowerRecovery
}
