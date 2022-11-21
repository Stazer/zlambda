use crate::leader::LeaderMessage;
use tokio::select;
use tokio::sync::mpsc;
use tracing::error;
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderClient {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_node_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderClient {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_node_sender: mpsc::Sender<LeaderMessage>,
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
