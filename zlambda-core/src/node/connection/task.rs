use zlambda_common::message::{MessageError, Message, MessageStreamReader, GuestToNodeRegisterRequestMessage, GuestToNodeRecoveryRequestMessage, MessageStreamWriter, ClientToNodeMessage, GuestToNodeMessage};
use tokio::{spawn};
use tracing::error;
use crate::node::NodeReference;
use crate::node::connection::{NodeConnectionAction, NodeConnectionClientRegistrationAction};
use crate::node::client::NodeClientTask;

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
            NodeConnectionAction::Stop | NodeConnectionAction::ConnectionClosed => {},
            NodeConnectionAction::Error(error) => error!("{}", error),
            NodeConnectionAction::ClientRegistration(action) => {
                let (message,) = action.into();

                NodeClientTask::new(
                    self.reader,
                    self.writer,
                    self.reference,
                    message,
                ).spawn()
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
            message => MessageError::UnexpectedMessage(message).into()
        }
    }

    async fn on_client_to_node_message(&mut self, message: ClientToNodeMessage) -> NodeConnectionAction {
        NodeConnectionClientRegistrationAction::new(message).into()
    }

    async fn on_guest_to_node_message(&mut self, message: GuestToNodeMessage) -> NodeConnectionAction {
        match message {
            GuestToNodeMessage::RegisterRequest(message) => self.on_guest_to_node_register_request_message(message).await,
            GuestToNodeMessage::RecoveryRequest(message) => self.on_guest_to_node_recovery_request_message(message).await,
        }
    }

    async fn on_guest_to_node_register_request_message(&mut self, _message: GuestToNodeRegisterRequestMessage) -> NodeConnectionAction {
        NodeConnectionAction::Stop
    }

    async fn on_guest_to_node_recovery_request_message(&mut self, _message: GuestToNodeRecoveryRequestMessage) -> NodeConnectionAction {
        NodeConnectionAction::Stop
    }
}
