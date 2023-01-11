use crate::node::member::NodeMemberReference;
use std::error::Error;
use zlambda_common::message::{ClientToNodeMessage, MessageError};

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
    reference: NodeMemberReference,
}

impl From<NodeConnectionFollowerRegistrationAction> for (NodeMemberReference,) {
    fn from(action: NodeConnectionFollowerRegistrationAction) -> Self {
        (action.reference,)
    }
}

impl From<NodeConnectionFollowerRegistrationAction> for NodeConnectionAction {
    fn from(action: NodeConnectionFollowerRegistrationAction) -> Self {
        NodeConnectionAction::FollowerRegistration(action)
    }
}

impl NodeConnectionFollowerRegistrationAction {
    pub fn new(reference: NodeMemberReference) -> Self {
        Self { reference }
    }

    pub fn reference(&self) -> &NodeMemberReference {
        &self.reference
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeConnectionFollowerHandshakeAction {
    reference: NodeMemberReference,
}

impl From<NodeConnectionFollowerHandshakeAction> for (NodeMemberReference,) {
    fn from(action: NodeConnectionFollowerHandshakeAction) -> Self {
        (action.reference,)
    }
}

impl From<NodeConnectionFollowerHandshakeAction> for NodeConnectionAction {
    fn from(action: NodeConnectionFollowerHandshakeAction) -> Self {
        NodeConnectionAction::FollowerHandshake(action)
    }
}

impl NodeConnectionFollowerHandshakeAction {
    pub fn new(reference: NodeMemberReference) -> Self {
        Self { reference }
    }

    pub fn reference(&self) -> &NodeMemberReference {
        &self.reference
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeConnectionFollowerRecoveryAction {
    reference: NodeMemberReference,
}

impl From<NodeConnectionFollowerRecoveryAction> for (NodeMemberReference,) {
    fn from(action: NodeConnectionFollowerRecoveryAction) -> Self {
        (action.reference,)
    }
}

impl From<NodeConnectionFollowerRecoveryAction> for NodeConnectionAction {
    fn from(action: NodeConnectionFollowerRecoveryAction) -> Self {
        NodeConnectionAction::FollowerRecovery(action)
    }
}

impl NodeConnectionFollowerRecoveryAction {
    pub fn new(reference: NodeMemberReference) -> Self {
        Self { reference }
    }

    pub fn reference(&self) -> &NodeMemberReference {
        &self.reference
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
    FollowerHandshake(NodeConnectionFollowerHandshakeAction),
    FollowerRecovery(NodeConnectionFollowerRecoveryAction),
}

impl From<MessageError> for NodeConnectionAction {
    fn from(error: MessageError) -> Self {
        Self::Error(Box::new(error))
    }
}
