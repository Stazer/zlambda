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
    leader_sender: mpsc::Sender<FollowerMessage>,
}

impl FollowerClient {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_sender: mpsc::Sender<FollowerMessage>,
    ) -> Self {
        Self {
            reader,
            writer,
            leader_sender,
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
                        Message::Client(client_message) => {
                            match client_message {
                                ClientMessage::InitializeModuleRequest => self.initialize_module().await,
                                message => {
                                    error!("Unhandled message {:?}", message);
                                    break;
                                }
                            }
                        }
                        message => {
                            error!("Unhandled message {:?}", message);
                            break;
                        }
                    }
                }
            )
        }
    }

    async fn initialize_module(&mut self) {
        //let (result_sender, result_receiver) = oneshot::channel();

        /*self.leader_sender
            .send(FollowerMessage::InitializeModule { result_sender })
            .await;

        result_receiver.await;*/
    }
}
