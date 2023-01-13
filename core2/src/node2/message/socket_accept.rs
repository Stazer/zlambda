use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::oneshot;

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
