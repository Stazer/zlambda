use tokio::sync::oneshot;
use crate::node::NodeMessage;

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

impl NodePingMessage {
    pub fn new(sender: oneshot::Sender<()>) -> Self {
        Self { sender }
    }

    pub fn sender(&self) -> &oneshot::Sender<()> {
        &self.sender
    }
}
