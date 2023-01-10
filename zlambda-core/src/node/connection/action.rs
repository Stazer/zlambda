use std::error::Error;
use zlambda_common::message::{MessageError, ClientToNodeMessage};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeConnectionClientRegistrationAction {
    message: ClientToNodeMessage,
}

impl From<NodeConnectionClientRegistrationAction> for (ClientToNodeMessage,) {
    fn from(action: NodeConnectionClientRegistrationAction) -> Self {
        (action.message,)
    }
}

impl From<NodeConnectionClientRegistrationAction> for NodeConnectionAction {
    fn from(action: NodeConnectionClientRegistrationAction) -> Self {
        NodeConnectionAction::ClientRegistration(action)
    }
}

impl NodeConnectionClientRegistrationAction {
    pub fn new(
        message: ClientToNodeMessage,
    ) -> Self {
        Self {
            message,
        }
    }

    pub fn message(&self) -> &ClientToNodeMessage {
        &self.message
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeConnectionAction {
    Stop,
    ConnectionClosed,
    Error(Box<dyn Error + Send>),
    ClientRegistration(NodeConnectionClientRegistrationAction),
}

impl From<MessageError> for NodeConnectionAction {
    fn from(error: MessageError) -> Self {
        Self::Error(Box::new(error))
    }
}
