use crate::node::member::NodeMemberReference;
use crate::node::{NodeFollowerRegistrationAttemptOutput, NodeId, NodeFollowerRegistrationAttemptError};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::oneshot;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollowerRegistrationAttemptMessage {
    address: SocketAddr,
    sender: oneshot::Sender<Result<NodeFollowerRegistrationAttemptOutput, NodeFollowerRegistrationAttemptError>>,
}

impl From<NodeFollowerRegistrationAttemptMessage>
    for (
        SocketAddr,
        oneshot::Sender<Result<NodeFollowerRegistrationAttemptOutput, NodeFollowerRegistrationAttemptError>>,
    )
{
    fn from(message: NodeFollowerRegistrationAttemptMessage) -> Self {
        (message.address, message.sender)
    }
}

impl NodeFollowerRegistrationAttemptMessage {
    pub fn new(
        address: SocketAddr,
        sender: oneshot::Sender<
            Result<NodeFollowerRegistrationAttemptOutput, NodeFollowerRegistrationAttemptError>,
        >,
    ) -> Self {
        Self { address, sender }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn sender(
        &self,
    ) -> &oneshot::Sender<Result<NodeFollowerRegistrationAttemptOutput, NodeFollowerRegistrationAttemptError>>
    {
        &self.sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollowerRegistrationAcknowledgementMessage {
    node_id: NodeId,
    node_member_reference: NodeMemberReference,
}

impl From<NodeFollowerRegistrationAcknowledgementMessage> for (NodeId, NodeMemberReference) {
    fn from(message: NodeFollowerRegistrationAcknowledgementMessage) -> Self {
        (message.node_id, message.node_member_reference)
    }
}

impl NodeFollowerRegistrationAcknowledgementMessage {
    pub fn new(node_id: NodeId, node_member_reference: NodeMemberReference) -> Self {
        Self {
            node_id,
            node_member_reference,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn node_member_reference(&self) -> &NodeMemberReference {
        &self.node_member_reference
    }
}
