use crate::follower::client::FollowerClientBuilder;
use crate::follower::connection::{
    FollowerConnectionClientRegistrationResult, FollowerConnectionResult,
};
use crate::follower::FollowerHandle;
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, FollowerToGuestRecoveryNotALeaderResponseMessage, FollowerToGuestMessage,
    FollowerToGuestRegisterNotALeaderResponseMessage, GuestToNodeRecoveryRequestMessage,
    GuestToNodeMessage, GuestToNodeRegisterRequestMessage, Message, MessageError,
    MessageStreamReader, MessageStreamWriter, FollowerToFollowerMessage,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerConnectionTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    follower_handle: FollowerHandle,
}

impl FollowerConnectionTask {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        follower_handle: FollowerHandle,
    ) -> Self {
        Self {
            reader,
            writer,
            follower_handle,
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.main().await;
        });
    }

    async fn main(mut self) {
        match self.select().await {
            FollowerConnectionResult::Stop | FollowerConnectionResult::ConnectionClosed => {}
            FollowerConnectionResult::Error(error) => error!("{}", error),
            FollowerConnectionResult::ClientRegistration(result) => {
                let (message,) = result.into();

                FollowerClientBuilder::default()
                    .task(
                        self.reader.into(),
                        self.writer.into(),
                        self.follower_handle,
                        message,
                    )
                    .await
                    .spawn()
            }
        }
    }

    async fn select(&mut self) -> FollowerConnectionResult {
        select!(
            result = self.reader.read() => {
                let message = match result {
                    Ok(None) => return FollowerConnectionResult::ConnectionClosed,
                    Ok(Some(message)) => message,
                    Err(error) => return FollowerConnectionResult::Error(Box::new(error)),
                };

                match message {
                    Message::GuestToNode(message) =>
                        self.on_guest_to_node_message(message).await,
                    Message::ClientToNode(message) =>
                        self.on_client_to_node_message(message).await,
                    Message::FollowerToFollower(message) =>
                        self.on_follower_to_follower(message).await,
                    message => MessageError::UnexpectedMessage(message).into(),
                }
            }
        )
    }

    async fn on_client_to_node_message(
        &mut self,
        message: ClientToNodeMessage,
    ) -> FollowerConnectionResult {
        FollowerConnectionResult::ClientRegistration(
            FollowerConnectionClientRegistrationResult::new(message),
        )
    }

    async fn on_guest_to_node_message(
        &mut self,
        message: GuestToNodeMessage,
    ) -> FollowerConnectionResult {
        match message {
            GuestToNodeMessage::RegisterRequest(message) => {
                self.on_guest_to_node_register_request_message(message)
                    .await
            }
            GuestToNodeMessage::RecoveryRequest(message) => {
                self.on_guest_to_node_recovery_request_message(message)
                    .await
            }
        }
    }

    async fn on_guest_to_node_register_request_message(
        &mut self,
        _message: GuestToNodeRegisterRequestMessage,
    ) -> FollowerConnectionResult {
        let result = self
            .writer
            .write(Message::FollowerToGuest(
                FollowerToGuestMessage::RegisterNotALeaderResponse(
                    FollowerToGuestRegisterNotALeaderResponseMessage::new(
                        self.follower_handle.leader_address().await,
                    ),
                ),
            ))
            .await;

        if let Err(error) = result {
            return error.into();
        }

        FollowerConnectionResult::Stop
    }

    async fn on_guest_to_node_recovery_request_message(
        &mut self,
        _message: GuestToNodeRecoveryRequestMessage,
    ) -> FollowerConnectionResult {
        let result = self
            .writer
            .write(Message::FollowerToGuest(
                FollowerToGuestMessage::RecoveryNotALeaderResponse(
                    FollowerToGuestRecoveryNotALeaderResponseMessage::new(
                        self.follower_handle.leader_address().await,
                    ),
                ),
            ))
            .await;

        if let Err(error) = result {
            return error.into();
        }

        FollowerConnectionResult::Stop
    }

    async fn on_follower_to_follower(
        &mut self,
        message: FollowerToFollowerMessage,
    ) -> FollowerConnectionResult {
        FollowerConnectionResult::Stop
    }
}
