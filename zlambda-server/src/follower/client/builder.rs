use crate::follower::client::FollowerClientTask;
use crate::follower::FollowerHandle;
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, ClientToNodeMessageStreamReader, NodeToClientMessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowerClientBuilder {}

impl FollowerClientBuilder {
    pub async fn task(
        self,
        reader: ClientToNodeMessageStreamReader,
        writer: NodeToClientMessageStreamWriter,
        follower_handle: FollowerHandle,
        initial_message: ClientToNodeMessage,
    ) -> FollowerClientTask {
        FollowerClientTask::new(reader, writer, follower_handle, initial_message).await
    }
}
