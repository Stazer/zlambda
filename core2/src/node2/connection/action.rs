use crate::message::{ClientToNodeMessage, MessageError};
use crate::node::member::NodeMemberReference;
use crate::node::NodeId;
use std::error::Error;
use std::net::SocketAddr;

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
    pub fn new(message: ClientToNodeMessage) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &ClientToNodeMessage {
        &self.message
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeConnectionFollowerRegistrationAction {
    node_id: NodeId,
}

impl From<NodeConnectionFollowerRegistrationAction> for (NodeId,) {
    fn from(action: NodeConnectionFollowerRegistrationAction) -> Self {
        (action.node_id,)
    }
}

impl From<NodeConnectionFollowerRegistrationAction> for NodeConnectionAction {
    fn from(action: NodeConnectionFollowerRegistrationAction) -> Self {
        NodeConnectionAction::FollowerRegistration(action)
    }
}

impl NodeConnectionFollowerRegistrationAction {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeConnectionAction {
    Stop,
    ConnectionClosed,
    Error(Box<dyn Error + Send>),
    ClientRegistration(NodeConnectionClientRegistrationAction),
    FollowerRegistration(NodeConnectionFollowerRegistrationAction),
}

impl From<MessageError> for NodeConnectionAction {
    fn from(error: MessageError) -> Self {
        Self::Error(Box::new(error))
    }
}
