use crate::leader::LeaderMessage;
use std::error::Error;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use zlambda_common::dispatch::DispatchId;
use zlambda_common::message::{ClientMessage, Message, MessageStreamReader, MessageStreamWriter};
use zlambda_common::module::{ModuleEventDispatchPayload, ModuleId};

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
                                ClientMessage::InitializeRequest => self.initialize().await.expect(""),
                                ClientMessage::Append(id, bytes) => self.append(id, bytes).await.expect(""),
                                ClientMessage::LoadRequest(id) => self.load(id).await.expect(""),
                                ClientMessage::DispatchRequest(module_id, dispatch_id, payload) => self.dispatch(module_id, dispatch_id, payload).await.expect(""),
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

    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();

        self.leader_sender
            .send(LeaderMessage::Initialize(sender))
            .await?;

        self.writer
            .write(&Message::Client(ClientMessage::InitializeResponse(
                receiver.await?,
            )))
            .await?;

        Ok(())
    }

    async fn append(&mut self, id: ModuleId, chunk: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();

        self.leader_sender
            .send(LeaderMessage::Append(id, chunk, sender))
            .await?;

        receiver.await?;

        Ok(())
    }

    async fn load(&mut self, id: ModuleId) -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();

        self.leader_sender
            .send(LeaderMessage::Load(id, sender))
            .await?;

        self.writer
            .write(&Message::Client(ClientMessage::LoadResponse(
                receiver.await?,
            )))
            .await?;

        Ok(())
    }

    async fn dispatch(
        &mut self,
        module_id: ModuleId,
        dispatch_id: DispatchId,
        payload: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();

        self.leader_sender
            .send(LeaderMessage::Dispatch(module_id, payload, sender))
            .await?;

        let result = receiver.await?;

        self.writer
            .write(&Message::Client(ClientMessage::DispatchResponse(
                dispatch_id,
                result,
            )))
            .await?;

        Ok(())
    }
}
