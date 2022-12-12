use crate::leader::LeaderMessage;
use std::error::Error;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use zlambda_common::dispatch::DispatchId;
use zlambda_common::message::{
    ClientToNodeMessage, ClientToNodeMessageStreamReader, NodeToClientMessage,
    NodeToClientMessageStreamWriter,
};
use zlambda_common::module::ModuleId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderClient {
    reader: ClientToNodeMessageStreamReader,
    writer: NodeToClientMessageStreamWriter,
    leader_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderClient {
    pub async fn new(
        reader: ClientToNodeMessageStreamReader,
        writer: NodeToClientMessageStreamWriter,
        leader_sender: mpsc::Sender<LeaderMessage>,
        initial_message: ClientToNodeMessage,
    ) -> Self {
        let mut leader_client = Self {
            reader,
            writer,
            leader_sender,
        };

        leader_client.handle_message(initial_message).await;

        leader_client
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

                    self.handle_message(message).await;
                }
            )
        }
    }

    async fn handle_message(&mut self, message: ClientToNodeMessage) {
        match message {
            ClientToNodeMessage::InitializeRequest => self.initialize().await.expect(""),
            ClientToNodeMessage::Append { module_id, bytes } => {
                self.append(module_id, bytes).await.expect("")
            }
            ClientToNodeMessage::LoadRequest { module_id } => self.load(module_id).await.expect(""),
            ClientToNodeMessage::DispatchRequest {
                module_id,
                dispatch_id,
                payload,
            } => self
                .dispatch(module_id, dispatch_id, payload)
                .await
                .expect(""),
        }
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();

        self.leader_sender
            .send(LeaderMessage::Initialize(sender))
            .await?;

        let module_id = receiver.await?;

        self.writer
            .write(NodeToClientMessage::InitializeResponse { module_id })
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

    async fn load(&mut self, module_id: ModuleId) -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();

        self.leader_sender
            .send(LeaderMessage::Load(module_id, sender))
            .await?;

        self.writer
            .write(NodeToClientMessage::LoadResponse {
                module_id,
                result: Ok(receiver.await.map(|x| ())?),
            })
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
            .write(NodeToClientMessage::DispatchResponse {
                dispatch_id,
                result,
            })
            .await?;

        Ok(())
    }
}
