use crate::node::client::NodeClientTask;
use crate::node::connection::{
    NodeConnectionAction, NodeConnectionClientRegistrationAction,
    NodeConnectionFollowerRegistrationAction,
};
use crate::node::NodeReference;
use tokio::spawn;
use tracing::error;
use zlambda_common::error::FollowerRegistrationError;
use zlambda_common::message::{
    ClientToNodeMessage, FollowerToGuestMessage, FollowerToGuestRegisterNotALeaderResponseMessage,
    GuestToNodeMessage, GuestToNodeRecoveryRequestMessage, GuestToNodeRegisterRequestMessage,
    Message, MessageError, MessageStreamReader, MessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeConnectionTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    reference: NodeReference,
}

impl NodeConnectionTask {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        reference: NodeReference,
    ) -> Self {
        Self {
            reader,
            writer,
            reference,
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        match self.select().await {
            NodeConnectionAction::Stop | NodeConnectionAction::ConnectionClosed => {}
            NodeConnectionAction::Error(error) => error!("{}", error),
            NodeConnectionAction::ClientRegistration(action) => {
                let (message,) = action.into();

                NodeClientTask::new(self.reader, self.writer, self.reference, message).spawn()
            }
            NodeConnectionAction::FollowerRegistration(action) => {}
            NodeConnectionAction::FollowerHandshake(action) => {}
            NodeConnectionAction::FollowerRecovery(action) => {}
        }
    }

    async fn select(&mut self) -> NodeConnectionAction {
        let message = match self.reader.read().await {
            Err(error) => return error.into(),
            Ok(None) => return NodeConnectionAction::ConnectionClosed,
            Ok(Some(message)) => message,
        };

        self.on_message(message).await
    }

    async fn on_message(&mut self, message: Message) -> NodeConnectionAction {
        match message {
            Message::ClientToNode(message) => self.on_client_to_node_message(message).await,
            Message::GuestToNode(message) => self.on_guest_to_node_message(message).await,
            message => MessageError::UnexpectedMessage(message).into(),
        }
    }

    async fn on_client_to_node_message(
        &mut self,
        message: ClientToNodeMessage,
    ) -> NodeConnectionAction {
        NodeConnectionClientRegistrationAction::new(message).into()
    }

    async fn on_guest_to_node_message(
        &mut self,
        message: GuestToNodeMessage,
    ) -> NodeConnectionAction {
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
    ) -> NodeConnectionAction {
        let (address,) = message.into();

        let member_reference = match self.reference.register_follower(address).await {
            Err(FollowerRegistrationError::NotALeader(error)) => {
                let result = self
                    .writer
                    .write(
                        FollowerToGuestMessage::RegisterNotALeaderResponse(
                            FollowerToGuestRegisterNotALeaderResponseMessage::new(
                                *error.leader_address(),
                            ),
                        )
                        .into(),
                    )
                    .await;

                return match result {
                    Err(error) => error.into(),
                    Ok(()) => NodeConnectionAction::Stop,
                };
            }
            Ok(member_reference) => member_reference,
        };

        NodeConnectionFollowerRegistrationAction::new(member_reference).into()
    }

    async fn on_guest_to_node_recovery_request_message(
        &mut self,
        _message: GuestToNodeRecoveryRequestMessage,
    ) -> NodeConnectionAction {
        NodeConnectionAction::Stop
    }
}
