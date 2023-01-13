use crate::channel::{DoReceive, DoSend};
use crate::message::{MessageStreamReader, MessageStreamWriter};
use crate::node::member::NodeMemberMessage;
use tokio::sync::{mpsc, oneshot};

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
