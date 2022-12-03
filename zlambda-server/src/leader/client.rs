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
                            break
                        }
                        Ok(Some(message)) => message,
                        Err(error) => {
                            error!("{:?}", error);
                            break
                        }
                    };

                    match message {
                        Message::Client(client_message) => {
                            match client_message {
                                ClientMessage::InitializeRequest => self.initialize().await,
                                ClientMessage::Append(id, bytes) => self.append(id, bytes).await,
                                ClientMessage::LoadRequest(id) => self.load(id).await,
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

    async fn initialize(&mut self) {
        let (sender, receiver) = oneshot::channel();

        let result = self
            .leader_sender
            .send(LeaderMessage::Initialize(sender))
            .await;

        if let Err(error) = result {
            error!("{}", error);
            return;
        }

        let id = match receiver.await {
            Err(error) => {
                error!("{}", error);
                return;
            }
            Ok(id) => id,
        };

        let message = Message::Client(ClientMessage::InitializeResponse(id));

        if let Err(error) = self.writer.write(&message).await {
            error!("{}", error);
        }
    }

    async fn append(&mut self, id: ModuleId, chunk: Vec<u8>) {
        let (sender, receiver) = oneshot::channel();

        let result = self
            .leader_sender
            .send(LeaderMessage::Append(id, chunk, sender))
            .await;

        receiver.await;

        if let Err(error) = result {
            error!("{}", error);
        }
    }

    async fn load(&mut self, id: ModuleId) {
        let (sender, receiver) = oneshot::channel();

        let result = self
            .leader_sender
            .send(LeaderMessage::Load(id, sender)).await;

        let result = match receiver.await {
            Err(error) => {
                error!("{}", error);
                return;
            }
            Ok(result) => result
        };

        let message = Message::Client(ClientMessage::LoadResponse(result));

        if let Err(error) = self.writer.write(&message).await {
            error!("{}", error);
        }
    }
}
