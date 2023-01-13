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
}
