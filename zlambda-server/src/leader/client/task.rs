use crate::leader::client::LeaderClientResult;
use crate::leader::LeaderHandle;
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeAppendMessage, ClientToNodeDispatchRequestMessage,
    ClientToNodeInitializeRequestMessage, ClientToNodeLoadRequestMessage, ClientToNodeMessage,
    ClientToNodeMessageStreamReader, NodeToClientInitializeResponseMessage,
    NodeToClientLoadResponseMessage, NodeToClientMessage, NodeToClientMessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderClientTask {
    reader: ClientToNodeMessageStreamReader,
    writer: NodeToClientMessageStreamWriter,
    leader_handle: LeaderHandle,
}

impl LeaderClientTask {
    pub async fn new(
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

        leader_client
            .on_client_to_node_message(initial_message)
            .await;

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

                    self.on_client_to_node_message(message).await;
                }
            )
        }
    }

    async fn on_client_to_node_message(&mut self, message: ClientToNodeMessage) {
        match message {
            ClientToNodeMessage::InitializeRequest(message) => {
                self.on_client_to_node_initialize_request_message(message)
                    .await
            }
            ClientToNodeMessage::Append(message) => {
                self.on_client_to_node_append_message(message).await
            }
            ClientToNodeMessage::LoadRequest(message) => {
                self.on_client_to_node_load_request_message(message).await
            }
            ClientToNodeMessage::DispatchRequest(message) => {
                self.on_client_to_node_dispatch_request_message(message)
                    .await
            }
        };
    }

    async fn on_client_to_node_initialize_request_message(
        &mut self,
        _message: ClientToNodeInitializeRequestMessage,
    ) -> LeaderClientResult {
        let module_id = self.leader_handle.initialize().await;

        if self
            .writer
            .write(NodeToClientMessage::InitializeResponse(
                NodeToClientInitializeResponseMessage::new(module_id),
            ))
            .await
            .is_err()
        {
            return LeaderClientResult::ConnectionClosed;
        }

        LeaderClientResult::Continue
    }

    async fn on_client_to_node_append_message(
        &mut self,
        message: ClientToNodeAppendMessage,
    ) -> LeaderClientResult {
        self.leader_handle
            .append(message.module_id(), message.bytes().clone())
            .await;

        LeaderClientResult::Continue
    }

    async fn on_client_to_node_load_request_message(
        &mut self,
        message: ClientToNodeLoadRequestMessage,
    ) -> LeaderClientResult {
        let result = self.leader_handle.load(message.module_id()).await;

        if self
            .writer
            .write(NodeToClientMessage::LoadResponse(
                NodeToClientLoadResponseMessage::new(message.module_id(), result.map(|_| ())),
            ))
            .await
            .is_err()
        {
            return LeaderClientResult::ConnectionClosed;
        }

        LeaderClientResult::Continue
    }

    async fn on_client_to_node_dispatch_request_message(
        &mut self,
        message: ClientToNodeDispatchRequestMessage,
    ) -> LeaderClientResult {
        let result = self
            .leader_handle
            .dispatch(
                message.module_id(),
                message.payload().clone(),
                message.target_node_id(),
            )
            .await;

        if self
            .writer
            .write(NodeToClientMessage::DispatchResponse {
                dispatch_id: message.dispatch_id(),
                result,
            })
            .await
            .is_err()
        {
            return LeaderClientResult::ConnectionClosed;
        }

        LeaderClientResult::Continue
    }
}
