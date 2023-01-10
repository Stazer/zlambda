use crate::follower::FollowerHandle;
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, ClientToNodeMessageStreamReader, Message, MessageError,
    NodeToClientMessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

mod builder;
mod result;
mod task;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use builder::*;
pub use result::*;
pub use task::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerClientTask {
    reader: ClientToNodeMessageStreamReader,
    _writer: NodeToClientMessageStreamWriter,
    _follower_handle: FollowerHandle,
}

impl FollowerClientTask {
    pub async fn new(
        reader: ClientToNodeMessageStreamReader,
        writer: NodeToClientMessageStreamWriter,
        follower_handle: FollowerHandle,
        initial_message: ClientToNodeMessage,
    ) -> Self {
        let mut follower_client = Self {
            reader,
            _writer: writer,
            _follower_handle: follower_handle,
        };

        follower_client
            .on_client_to_node_message(initial_message)
            .await;

        follower_client
    }

    pub fn spawn(self) {
        spawn(async move {
            self.main().await;
        });
    }

    async fn main(mut self) {
        loop {
            match self.select().await {
                FollowerClientResult::Continue => {}
                FollowerClientResult::Stop | FollowerClientResult::ConnectionClosed => break,
                FollowerClientResult::Error(error) => {
                    error!("{}", error);
                    break;
                }
            }
        }
    }

    async fn select(&mut self) -> FollowerClientResult {
        select!(
            result = self.reader.read() => {
                let message = match result {
                    Err(error) => return error.into(),
                    Ok(None) => return FollowerClientResult::ConnectionClosed,
                    Ok(Some(message)) => message,
                };

                self.on_client_to_node_message(message).await
            }
        )
    }

    async fn on_client_to_node_message(
        &mut self,
        message: ClientToNodeMessage,
    ) -> FollowerClientResult {
        MessageError::UnexpectedMessage(Message::ClientToNode(message)).into()
    }
}
