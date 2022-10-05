use crate::cluster::ConnectionId;
use actix::Message;
use tokio::net::TcpStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActorRemoveConnectionMessage {
    connection_id: ConnectionId,
}

impl From<NodeActorRemoveConnectionMessage> for (ConnectionId,) {
    fn from(message: NodeActorRemoveConnectionMessage) -> Self {
        (message.connection_id,)
    }
}

impl Message for NodeActorRemoveConnectionMessage {
    type Result = ();
}

impl NodeActorRemoveConnectionMessage {
    pub fn new(connection_id: ConnectionId) -> Self {
        Self { connection_id }
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActorRegisterMessage {
    stream: TcpStream,
}

impl From<NodeActorRegisterMessage> for (TcpStream,) {
    fn from(message: NodeActorRegisterMessage) -> Self {
        (message.stream,)
    }
}

impl Message for NodeActorRegisterMessage {
    type Result = ();
}

impl NodeActorRegisterMessage {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
