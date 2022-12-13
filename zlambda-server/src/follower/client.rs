use crate::follower::FollowerMessage;
use tokio::select;
use tokio::sync::mpsc;
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, ClientToNodeMessageStreamReader, NodeToClientMessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerClient {
    reader: ClientToNodeMessageStreamReader,
    _writer: NodeToClientMessageStreamWriter,
    _follower_sender: mpsc::Sender<FollowerMessage>,
}

impl FollowerClient {
    pub async fn new(
        reader: ClientToNodeMessageStreamReader,
        writer: NodeToClientMessageStreamWriter,
        follower_sender: mpsc::Sender<FollowerMessage>,
        initial_message: ClientToNodeMessage,
    ) -> Self {
        let mut follower_client = Self {
            reader,
            _writer: writer,
            _follower_sender: follower_sender,
        };

        follower_client.handle_message(initial_message).await;

        follower_client
    }

    pub async fn run(mut self) {
        loop {
            select!(
                read_result = self.reader.read() => {
                    let message = match read_result {
                        Ok(None) => {
                            break
                        }
                        Ok(Some(message)) => message,
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                    };

                    self.handle_message(message).await;
                }
            )
        }
    }

    async fn handle_message(&mut self, message: ClientToNodeMessage) {
        error!("Unhandled message {:?}", message);
    }
}
