use crate::message::{
    ClientToNodeMessage, FollowerToGuestMessage, FollowerToGuestRegisterNotALeaderResponseMessage,
    GuestToNodeMessage, GuestToNodeRecoveryRequestMessage, GuestToNodeRegisterRequestMessage,
    Message, MessageError, MessageStreamReader, MessageStreamWriter,
};
use crate::node::client::NodeClientTask;
use crate::node::connection::{
    NodeConnectionAction, NodeConnectionClientRegistrationAction,
};
use crate::node::{NodeFollowerRegistrationError, NodeReference};
use tokio::spawn;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeConnectionTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    node_reference: NodeReference,
}

impl NodeConnectionTask {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        node_reference: NodeReference,
    ) -> Self {
        Self {
            reader,
            writer,
            node_reference,
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

                NodeClientTask::new(self.reader, self.writer, self.node_reference, message).spawn()
            }
            NodeConnectionAction::FollowerRegistration(action) => {
                let (address,) = action.into();

                let (leader_address, _, mut writer) = match self.node_reference.register_follower(address, self.reader, self.writer).await {
                    Ok(()) => return,
                    Err(NodeFollowerRegistrationError::NotALeader(error)) => error.into(),
                };

                let result =
                    writer
                    .write(
                        FollowerToGuestMessage::RegisterNotALeaderResponse(
                            FollowerToGuestRegisterNotALeaderResponseMessage::new(
                                leader_address,
                            ),
                        )
                        .into(),
                    )
                    .await;

                if let Err(error) = result {
                    error!("{}", error);
                }
            }
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
        NodeConnectionAction::Stop
        /*let (node_member_reference,) = match self.reference.register_follower(address).await {
            Err(NodeFollowerRegistrationError::NotALeader(error)) => {
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
            Ok(node_member_reference) => node_member_reference.into(),
        };*/

        //NodeConnectionFollowerRegistrationResult::new(node_member_reference).into()
    }

    async fn on_guest_to_node_recovery_request_message(
        &mut self,
        _message: GuestToNodeRecoveryRequestMessage,
    ) -> NodeConnectionAction {
        NodeConnectionAction::Stop
    }
}
