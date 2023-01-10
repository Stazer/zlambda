use crate::node::sibling::NodeSiblingAction;
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeSiblingTask {}

impl NodeSiblingTask {
    pub fn new() -> Self {
        Self {}
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        loop {
            match self.select().await {
                NodeSiblingAction::Continue => {}
                NodeSiblingAction::Stop => break,
                NodeSiblingAction::Error(error) => {
                    error!("{}", error);
                    break;
                }
            }
        }
    }

    async fn select(&mut self) -> NodeSiblingAction {
        NodeSiblingAction::Continue
    }
}
