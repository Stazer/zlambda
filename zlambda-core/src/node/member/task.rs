use crate::node::member::{NodeMemberAction, NodeMemberMessage, NodeMemberReference};
use crate::node::NodeId;
use crate::node::NodeReference;
use crate::message::{MessageStreamReader, MessageStreamWriter};
use tokio::spawn;
use tokio::sync::mpsc;
use tracing::error;

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
    pub fn new(node_id: NodeId, node_reference: NodeReference) -> Self {
        let (sender, receiver) = mpsc::channel(16);

        Self {
            node_id,
            node_reference,
            sender,
            receiver,
            reader: None,
            writer: None,
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
        NodeMemberAction::Continue
    }
}
