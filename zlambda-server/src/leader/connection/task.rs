use crate::leader::client::LeaderClientBuilder;
use crate::leader::connection::{
    LeaderConnectionClientRegistrationResult, LeaderConnectionFollowerRecoveryResult,
    LeaderConnectionFollowerRegistrationResult, LeaderConnectionResult,
};
use crate::leader::follower::LeaderFollowerBuilder;
use crate::leader::LeaderHandle;
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, GuestToNodeRecoveryRequestMessage, GuestToNodeMessage,
    GuestToNodeRegisterRequestMessage, LeaderToGuestRecoveryErrorResponseMessage,
    LeaderToGuestMessage, LeaderToGuestRegisterOkResponseMessage, Message, MessageError,
    MessageStreamReader, MessageStreamWriter,
};
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderConnectionTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_handle: LeaderHandle,
}

impl LeaderConnectionTask {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_handle: LeaderHandle,
    ) -> Self {
        Self {
            reader,
            writer,
            leader_handle,
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.main().await;
        });
    }

    async fn main(mut self) {
        match self.select().await {
            LeaderConnectionResult::Stop | LeaderConnectionResult::ConnectionClosed => {}
            LeaderConnectionResult::Error(error) => error!("{:?}", error),
            LeaderConnectionResult::ClientRegistration(message) => {
                let (message,) = message.into();

                LeaderClientBuilder::new()
                    .task(
                        self.reader.into(),
                        self.writer.into(),
                        self.leader_handle,
                        message,
                    )
                    .await
                    .spawn()
            }
            LeaderConnectionResult::FollowerRecovery(message) => {
                let (
                    term,
                    acknowledging_log_entry_data,
                    last_committed_log_entry_id,
                    follower_handle,
                ) = message.into();

                let result = follower_handle
                    .recovery(
                        self.reader.into(),
                        self.writer.into(),
                        term,
                        acknowledging_log_entry_data,
                        last_committed_log_entry_id,
                    )
                    .await;

                if let Err(error) = result {
                    error!("{:?}", error);
                }
            }
            LeaderConnectionResult::FollowerRegistration(message) => {
                let (node_id, builder) = message.into();

                builder
                    .build(
                        node_id,
                        Some(self.reader.into()),
                        Some(self.writer.into()),
                        self.leader_handle,
                    )
                    .spawn()
            }
        }
    }

    async fn select(&mut self) -> LeaderConnectionResult {
        select!(
            result = self.reader.read() => {
                let message = match result {
                    Ok(None) => return LeaderConnectionResult::ConnectionClosed,
                    Ok(Some(message)) => message,
                    Err(error) => return error.into(),
                };

                self.on_message(message).await
            }
        )
    }

    async fn on_message(&mut self, message: Message) -> LeaderConnectionResult {
        match message {
            Message::GuestToNode(message) => self.on_guest_to_node_message(message).await,
            Message::ClientToNode(message) => self.on_client_to_node_message(message).await,
            message => MessageError::UnexpectedMessage(message).into(),
        }
    }

    async fn on_guest_to_node_message(
        &mut self,
        message: GuestToNodeMessage,
    ) -> LeaderConnectionResult {
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
        message: GuestToNodeRegisterRequestMessage,
    ) -> LeaderConnectionResult {
        let builder = LeaderFollowerBuilder::new();

        let (node_id, leader_node_id, term, addresses) = self
            .leader_handle
            .register(*message.address(), builder.handle())
            .await;

        let result = self
            .writer
            .write(Message::LeaderToGuest(
                LeaderToGuestMessage::RegisterOkResponse(
                    LeaderToGuestRegisterOkResponseMessage::new(
                        node_id,
                        leader_node_id,
                        addresses,
                        term,
                    ),
                ),
            ))
            .await;

        if let Err(error) = result {
            return error.into();
        }

        LeaderConnectionResult::FollowerRegistration(
            LeaderConnectionFollowerRegistrationResult::new(node_id, builder),
        )
    }

    async fn on_guest_to_node_recovery_request_message(
        &mut self,
        message: GuestToNodeRecoveryRequestMessage,
    ) -> LeaderConnectionResult {
        let (term, acknowledging_log_entry_data, last_committed_log_entry_id, follower_handle) =
            match self
                .leader_handle
                .recovery(message.node_id(), *message.address())
                .await
            {
                Ok(result) => result,
                Err(message) => {
                    let result = self
                        .writer
                        .write(Message::LeaderToGuest(
                            LeaderToGuestMessage::RecoveryErrorResponse(
                                LeaderToGuestRecoveryErrorResponseMessage::new(message),
                            ),
                        ))
                        .await;

                    if let Err(error) = result {
                        return error.into();
                    }

                    return LeaderConnectionResult::Stop;
                }
            };

        LeaderConnectionResult::FollowerRecovery(LeaderConnectionFollowerRecoveryResult::new(
            term,
            acknowledging_log_entry_data,
            last_committed_log_entry_id,
            follower_handle,
        ))
    }

    async fn on_client_to_node_message(
        &mut self,
        message: ClientToNodeMessage,
    ) -> LeaderConnectionResult {
        LeaderConnectionResult::ClientRegistration(LeaderConnectionClientRegistrationResult::new(
            message,
        ))
    }
}
