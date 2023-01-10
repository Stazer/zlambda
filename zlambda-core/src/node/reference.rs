use crate::node::{NodeMessage, NodePingMessage};
use tokio::sync::{mpsc, oneshot};
use zlambda_common::channel::{DoReceive, DoSend};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct NodeReference {
    sender: mpsc::Sender<NodeMessage>,
}

impl NodeReference {
    pub fn new(sender: mpsc::Sender<NodeMessage>) -> Self {
        Self { sender }
    }

    pub async fn ping(&self) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(NodePingMessage::new(sender).into())
            .await;

        receiver.do_receive().await
    }
}
