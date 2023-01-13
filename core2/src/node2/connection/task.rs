use crate::message::{
    ClientToNodeMessage, FollowerToGuestMessage, FollowerToGuestRegisterNotALeaderResponseMessage,
    GuestToNodeMessage, GuestToNodeRecoveryRequestMessage, GuestToNodeRegisterRequestMessage,
    LeaderToGuestMessage, LeaderToGuestRegisterOkResponseMessage, Message, MessageError,
    MessageStreamReader, MessageStreamWriter,
};
use crate::node::client::NodeClientTask;
use crate::node::connection::{
    NodeConnectionAction, NodeConnectionClientRegistrationAction,
    NodeConnectionFollowerRegistrationAction,
};
use crate::node::member::NodeMemberTask;
use crate::node::{NodeFollowerRegistrationAttemptError, NodeReference};
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

                NodeClientTask::new(self.node_reference, message, self.reader, self.writer).spawn()
            }
            NodeConnectionAction::FollowerRegistration(action) => {
                let (node_id,) = action.into();

                let task = NodeMemberTask::new(
                    node_id,
                    self.node_reference.clone(),
                    Some(self.reader),
                    Some(self.writer),
                );
                let reference = task.reference();

                task.spawn();

                self.node_reference
                    .acknowledge_follower_registration(node_id, reference);
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
        let (node_address,) = message.into();

        match self.node_reference.attempt_follower_registration(node_address).await {
            Ok(output) => {
                let (node_id, node_leader_id, node_socket_addresses, term) = output.into();

                let result = self
                    .writer
                    .write(
                        LeaderToGuestMessage::RegisterOkResponse(
                            LeaderToGuestRegisterOkResponseMessage::new(
                                node_id,
                                node_leader_id,
                                node_socket_addresses,
                                term,
                            ),
                        )
                        .into(),
                    )
                    .await;

                NodeConnectionFollowerRegistrationAction::new(node_id).into()
            }
            Err(NodeFollowerRegistrationAttemptError::NotALeader(error)) => {
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

                if let Err(error) = result {
                    // TODO remove node_address
                    return error.into();
                }

                match result {
                    Err(error) => error.into(),
                    Ok(()) => NodeConnectionAction::Stop,
                }
            }
        }
    }

    async fn on_guest_to_node_recovery_request_message(
        &mut self,
        _message: GuestToNodeRecoveryRequestMessage,
    ) -> NodeConnectionAction {
        NodeConnectionAction::Stop
    }
}
