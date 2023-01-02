use crate::leader::LeaderHandle;
use bytes::Bytes;
use std::error::Error;
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::dispatch::DispatchId;
use zlambda_common::message::{
    ClientToNodeMessage, ClientToNodeMessageStreamReader, NodeToClientMessage,
    NodeToClientMessageStreamWriter,
};
use zlambda_common::module::ModuleId;
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderClientBuilder {}

impl LeaderClientBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn task(
        self,
        reader: ClientToNodeMessageStreamReader,
        writer: NodeToClientMessageStreamWriter,
        leader_handle: LeaderHandle,
        initial_message: ClientToNodeMessage,
    ) -> LeaderClientTask {
        LeaderClientTask::new(reader, writer, leader_handle, initial_message).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderClientTask {
    reader: ClientToNodeMessageStreamReader,
    writer: NodeToClientMessageStreamWriter,
    leader_handle: LeaderHandle,
}

impl LeaderClientTask {
    async fn new(
        reader: ClientToNodeMessageStreamReader,
        writer: NodeToClientMessageStreamWriter,
        leader_handle: LeaderHandle,
        initial_message: ClientToNodeMessage,
    ) -> Self {
        let mut leader_client = Self {
            reader,
            writer,
            leader_handle,
        };

        leader_client.handle_message(initial_message).await;

        leader_client
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
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
                node_id,
            } => self
                .dispatch(module_id, dispatch_id, payload, node_id)
                .await
                .expect(""),
        }
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        let module_id = self.leader_handle.initialize().await;

        self.writer
            .write(NodeToClientMessage::InitializeResponse { module_id })
            .await?;

        Ok(())
    }

    async fn append(&mut self, id: ModuleId, bytes: Bytes) -> Result<(), Box<dyn Error>> {
        self.leader_handle.append(id, bytes).await;

        Ok(())
    }

    async fn load(&mut self, module_id: ModuleId) -> Result<(), Box<dyn Error>> {
        let result = self.leader_handle.load(module_id).await;

        self.writer
            .write(NodeToClientMessage::LoadResponse {
                module_id,
                result: Ok(result.map(|_x| ())?),
            })
            .await?;

        Ok(())
    }

    async fn dispatch(
        &mut self,
        module_id: ModuleId,
        dispatch_id: DispatchId,
        payload: Vec<u8>,
        node_id: Option<NodeId>,
    ) -> Result<(), Box<dyn Error>> {
        let result = self
            .leader_handle
            .dispatch(module_id, payload, node_id)
            .await;

        self.writer
            .write(NodeToClientMessage::DispatchResponse {
                dispatch_id,
                result,
            })
            .await?;

        Ok(())
    }
}
