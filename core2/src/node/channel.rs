use crate::channel::MessageSender;
use crate::node::NodeMessage;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct NodeMessageChannel {
    sender: MessageSender<NodeMessage>,
}

impl NodeMessageChannel {
    pub fn new(sender: MessageSender<NodeMessage>) -> Self {
        Self { sender }
    }
}
