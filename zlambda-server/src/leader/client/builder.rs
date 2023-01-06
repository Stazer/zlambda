use crate::leader::client::LeaderClientTask;
use crate::leader::LeaderHandle;
use zlambda_common::message::{
    ClientToNodeMessage, ClientToNodeMessageStreamReader, NodeToClientMessageStreamWriter,
};

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
