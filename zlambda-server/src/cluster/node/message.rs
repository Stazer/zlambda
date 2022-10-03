use crate::cluster::NodeConnectionId;
use actix::Message;
use tokio::net::TcpStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActorRemoveConnectionMessage {
    id: NodeConnectionId,
}

impl From<NodeActorRemoveConnectionMessage> for (NodeConnectionId,) {
    fn from(message: NodeActorRemoveConnectionMessage) -> Self {
        (message.id,)
    }
}

impl Message for NodeActorRemoveConnectionMessage {
    type Result = ();
}

impl NodeActorRemoveConnectionMessage {
    pub fn new(id: NodeConnectionId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> NodeConnectionId {
        self.id
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
