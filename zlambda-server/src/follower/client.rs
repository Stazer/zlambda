use crate::follower::FollowerMessage;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use zlambda_common::message::{ClientMessage, Message, MessageStreamReader, MessageStreamWriter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerClient {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    follower_sender: mpsc::Sender<FollowerMessage>,
}

impl FollowerClient {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        follower_sender: mpsc::Sender<FollowerMessage>,
    ) -> Self {
        Self {
            reader,
            writer,
            follower_sender,
        }
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

                    match message {
                        message => {
                            error!("Unhandled message {:?}", message);
                            break;
                        }
                    }
                }
            )
        }
    }
}
