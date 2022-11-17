pub mod connection;
pub mod follower;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::node::NodeId;
use crate::read_write::{read_write_channel, ReadWriteReceiver, ReadWriteSender};
use follower::LeaderNodeFollower;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNode {
    sender: ReadWriteSender<LeaderNode>,
    receiver: ReadWriteReceiver<LeaderNode>,
    follower_senders: HashMap<NodeId, ReadWriteSender<LeaderNodeFollower>>,
}

impl LeaderNode {
    pub fn new() -> Self {
        let (sender, receiver) = read_write_channel();

        Self {
            sender,
            receiver,
            follower_senders: HashMap::default(),
        }
    }

    pub fn sender(&self) -> &ReadWriteSender<LeaderNode> {
        &self.sender
    }

    pub fn receiver_mut(&mut self) -> &mut ReadWriteReceiver<LeaderNode> {
        &mut self.receiver
    }

    pub fn follower_senders_mut(
        &mut self,
    ) -> &mut HashMap<NodeId, ReadWriteSender<LeaderNodeFollower>> {
        &mut self.follower_senders
    }
}
