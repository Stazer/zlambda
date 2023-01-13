use crate::channel::{DoReceive, DoSend};
use crate::node::member::NodeMemberReference;
use crate::node::{
    NodeFollowerRegistrationError, NodeFollowerRegistrationMessage,
    NodeMessage, NodePingMessage,
};
use crate::message::{MessageStreamReader, MessageStreamWriter};
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

    pub async fn register_follower(
        &self,
        address: SocketAddr,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
    ) -> Result<(), NodeFollowerRegistrationError> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(NodeFollowerRegistrationMessage::new(address, reader, writer, sender).into())
            .await;

        receiver.do_receive().await
    }
}
