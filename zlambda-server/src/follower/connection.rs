use crate::follower::client::FollowerClientBuilder;
use crate::follower::FollowerHandle;
use std::error::Error;
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::error::SimpleError;
use zlambda_common::message::{
    ClientToNodeMessage, FollowerToGuestHandshakeNotALeaderResponseMessage, FollowerToGuestMessage,
    FollowerToGuestRegisterNotALeaderResponseMessage, GuestToNodeMessage, Message,
    MessageStreamReader, MessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum FollowerConnectionTaskResult {
    Ok,
    SwitchToClient(ClientToNodeMessage),
    ConnectionClosed,
    Error(Box<dyn Error + Send>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowerConnectionBuilder {}

impl FollowerConnectionBuilder {
    pub fn task(
        self,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        follower_handle: FollowerHandle,
    ) -> FollowerConnectionTask {
        FollowerConnectionTask::new(reader, writer, follower_handle)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerConnectionTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    follower_handle: FollowerHandle,
}

impl FollowerConnectionTask {
    fn new(
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
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        loop {
            match self.select().await {
                FollowerConnectionTaskResult::Ok => {}
                FollowerConnectionTaskResult::SwitchToClient(message) => {
                    self.switch_to_client(message).await;
                    break;
                }
                FollowerConnectionTaskResult::ConnectionClosed => break,
                FollowerConnectionTaskResult::Error(error) => {
                    error!("{}", error);
                    break;
                }
            }
        }
    }

    async fn select(&mut self) -> FollowerConnectionTaskResult {
        select!(
            read_result = self.reader.read() => {
                let message = match read_result {
                    Ok(message) => message,
                    Err(error) => return FollowerConnectionTaskResult::Error(Box::new(error)),
                };

                match message {
                    None => return FollowerConnectionTaskResult::ConnectionClosed,
                    Some(Message::GuestToNode(message)) =>
                        self.on_guest_to_node_message(message).await,
                    Some(Message::ClientToNode(message)) =>
                        self.on_client_to_node_message(message).await,
                    Some(message) =>
                        FollowerConnectionTaskResult::Error(
                            Box::new(SimpleError::new(format!("Unhandled message {:?}", message)))
                        ),
                }
            }
        )
    }

    async fn on_client_to_node_message(
        &mut self,
        message: ClientToNodeMessage,
    ) -> FollowerConnectionTaskResult {
        FollowerConnectionTaskResult::SwitchToClient(message)
    }

    async fn on_guest_to_node_message(
        &mut self,
        message: GuestToNodeMessage,
    ) -> FollowerConnectionTaskResult {
        match message {
            GuestToNodeMessage::RegisterRequest { .. } => {
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
                    error!("{}", error);
                }
            }
            GuestToNodeMessage::HandshakeRequest { .. } => {
                let result = self
                    .writer
                    .write(Message::FollowerToGuest(
                        FollowerToGuestMessage::HandshakeNotALeaderResponse(
                            FollowerToGuestHandshakeNotALeaderResponseMessage::new(
                                self.follower_handle.leader_address().await,
                            ),
                        ),
                    ))
                    .await;

                if let Err(error) = result {
                    error!("{}", error);
                }
            }
        };

        FollowerConnectionTaskResult::Ok
    }

    async fn switch_to_client(self, initial_message: ClientToNodeMessage) {
        FollowerClientBuilder::new()
            .task(
                self.reader.into(),
                self.writer.into(),
                self.follower_handle,
                initial_message,
            )
            .await
            .spawn()
    }
}
