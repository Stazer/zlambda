use crate::channel::{DoReceive, DoSend};
use crate::message::{MessageStreamReader, MessageStreamWriter};
use crate::node::member::{
    NodeMemberAction,
    NodeMemberMessage, NodeMemberReference,
};
use crate::node::NodeId;
use crate::node::NodeReference;
use tokio::sync::mpsc;
use tokio::{select, spawn};
use tracing::{error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeMemberTask {
    node_id: NodeId,
    node_reference: NodeReference,
    reader: Option<MessageStreamReader>,
    writer: Option<MessageStreamWriter>,
    receiver: mpsc::Receiver<NodeMemberMessage>,
    sender: mpsc::Sender<NodeMemberMessage>,
}

impl NodeMemberTask {
    pub fn new(node_id: NodeId, node_reference: NodeReference, reader: Option<MessageStreamReader>, writer: Option<MessageStreamWriter>) -> Self {
        let (sender, receiver) = mpsc::channel(16);

        Self {
            node_id,
            node_reference,
            sender,
            receiver,
            reader,
            writer,
        }
    }

    pub fn reference(&self) -> NodeMemberReference {
        NodeMemberReference::new(self.sender.clone())
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        loop {
            match self.select().await {
                NodeMemberAction::Continue => {}
                NodeMemberAction::Stop => break,
                NodeMemberAction::Error(error) => {
                    error!("{}", error);
                    break;
                }
            }
        }
    }

    async fn select(&mut self) -> NodeMemberAction {
        select!(
            message = self.receiver.do_receive() => {
                self.on_node_member_message(message).await
            }
        )
    }

    async fn on_node_member_message(&mut self, message: NodeMemberMessage) -> NodeMemberAction {
        NodeMemberAction::Continue
    }
}
