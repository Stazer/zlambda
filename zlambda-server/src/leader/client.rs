use crate::leader::LeaderMessage;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use zlambda_common::message::{ClientMessage, Message, MessageStreamReader, MessageStreamWriter};
use zlambda_common::module::ModuleId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderClient {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderClient {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_sender: mpsc::Sender<LeaderMessage>,
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
                            continue
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
                                ClientMessage::AppendModuleChunk { id, bytes } => self.append_chunk(id, bytes).await,
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
        let (result_sender, result_receiver) = oneshot::channel();

        let result = self
            .leader_sender
            .send(LeaderMessage::InitializeModule { result_sender })
            .await;

        if let Err(error) = result {
            error!("{}", error);
            return;
        }

        let id = match result_receiver.await {
            Err(error) => {
                error!("{}", error);
                return;
            }
            Ok(id) => id,
        };

        let message = Message::Client(ClientMessage::InitializeModuleResponse { id });

        if let Err(error) = self.writer.write(&message).await {
            error!("{}", error);
        }
    }

    async fn append_chunk(&mut self, id: ModuleId, chunk: Vec<u8>) {
        let result = self
            .leader_sender
            .send(LeaderMessage::Append { id, chunk })
            .await;

        if let Err(error) = result {
            error!("{}", error);
            return;
        }
    }
}
