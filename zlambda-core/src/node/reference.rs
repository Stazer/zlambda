use crate::node::member::NodeMemberReference;
use crate::node::{NodeFollowerRegistrationMessage, NodeMessage, NodePingMessage};
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};
use zlambda_common::channel::{DoReceive, DoSend};
use zlambda_common::error::FollowerRegistrationError;

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

    pub async fn register_follower(
        &self,
        address: SocketAddr,
    ) -> Result<NodeMemberReference, FollowerRegistrationError> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(NodeFollowerRegistrationMessage::new(address, sender).into())
            .await;

        receiver.do_receive().await
    }
}
