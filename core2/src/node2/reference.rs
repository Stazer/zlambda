use crate::channel::{DoReceive, DoSend};
use crate::message::{MessageStreamReader, MessageStreamWriter};
use crate::node::member::NodeMemberReference;
use crate::node::{
    NodeFollowerRegistrationAttemptMessage, NodeFollowerRegistrationAttemptOutput, NodeFollowerRegistrationAttemptError,
    NodeFollowerRegistrationAcknowledgementMessage, NodeId, NodeMessage,
    NodePingMessage,
};
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct NodeReference {
    sender: mpsc::Sender<NodeMessage>,
}

impl NodeReference {
    pub fn new(sender: mpsc::Sender<NodeMessage>) -> Self {
        Self { sender }
    }

    pub async fn ping(&self) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(NodePingMessage::new(sender).into())
            .await;

        receiver.do_receive().await
    }

    pub async fn attempt_follower_registration(
        &self,
        address: SocketAddr,
    ) -> Result<NodeFollowerRegistrationAttemptOutput, NodeFollowerRegistrationAttemptError> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(NodeFollowerRegistrationAttemptMessage::new(address, sender).into())
            .await;

        receiver.do_receive().await
    }

    pub async fn acknowledge_follower_registration(
        &self,
        node_id: NodeId,
        node_member_reference: NodeMemberReference,
    ) {
        self.sender
            .do_send(
                NodeFollowerRegistrationAcknowledgementMessage::new(node_id, node_member_reference)
                    .into(),
            )
            .await;
    }
}
