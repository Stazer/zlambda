use crate::node::member::NodeMemberMessage;
use tokio::sync::mpsc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct NodeMemberReference {
    sender: mpsc::Sender<NodeMemberMessage>,
}

impl NodeMemberReference {
    pub fn new(sender: mpsc::Sender<NodeMemberMessage>) -> Self {
        Self { sender }
    }
}
