use crate::message::{ClientToNodeMessage, MessageStreamReader, MessageStreamWriter};
use crate::node::client::NodeClientAction;
use crate::node::NodeReference;
use tokio::spawn;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeClientTask {
    reference: NodeReference,
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
}

impl NodeClientTask {
    pub fn new(
        reference: NodeReference,
        initial_message: ClientToNodeMessage,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
    ) -> Self {
        Self {
            reader,
            writer,
            reference,
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        loop {
            match self.select().await {
                NodeClientAction::Continue => {}
                NodeClientAction::Stop | NodeClientAction::ConnectionClosed => break,
                NodeClientAction::Error(error) => {
                    error!("{}", error);
                    break;
                }
            }
        }
    }

    async fn select(&mut self) -> NodeClientAction {
        NodeClientAction::Continue
    }
}
