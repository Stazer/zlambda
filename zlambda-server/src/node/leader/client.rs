use crate::node::leader::LeaderNodeMessage;
use tokio::select;
use tokio::sync::mpsc;
use tracing::error;
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeClient {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_node_sender: mpsc::Sender<LeaderNodeMessage>,
}

impl LeaderNodeClient {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_node_sender: mpsc::Sender<LeaderNodeMessage>,
    ) -> Self {
        Self {
            reader,
            writer,
            leader_node_sender,
        }
    }

    pub async fn run(mut self) {
        loop {
            select!(
                read_result = self.reader.read() => {
                    let message = match read_result {
                        Ok(message) => message,
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                    };

                    match message {
                        None => continue,
                        Some(message) => {
                            error!("Unhandled message {:?}", message);
                            break
                        }
                    };
                }
            )
        }
    }
}
