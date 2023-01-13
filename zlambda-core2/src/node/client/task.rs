use crate::message::{ClientToNodeMessage, MessageStreamReader, MessageStreamWriter};
use crate::node::client::NodeClientAction;
use crate::node::NodeReference;
use tokio::spawn;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeClientTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    reference: NodeReference,
}

impl NodeClientTask {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        reference: NodeReference,
        initial_message: ClientToNodeMessage,
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
