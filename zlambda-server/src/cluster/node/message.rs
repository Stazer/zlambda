use crate::cluster::NodeConnectionId;
use actix::Message;

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
