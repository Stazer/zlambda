use crate::node::member::NodeMemberAction;
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeMemberTask {}

impl NodeMemberTask {
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
