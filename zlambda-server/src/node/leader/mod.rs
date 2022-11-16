pub mod connection;
pub mod follower;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::read_write::{read_write_channel, ReadWriteReceiver, ReadWriteSender};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNode {
    sender: ReadWriteSender<LeaderNode>,
    receiver: ReadWriteReceiver<LeaderNode>,
}

impl LeaderNode {
    pub fn new() -> Self {
        let (sender, receiver) = read_write_channel();

        Self { sender, receiver }
    }

    pub fn sender(&self) -> &ReadWriteSender<LeaderNode> {
        &self.sender
    }

    pub fn receiver_mut(&mut self) -> &mut ReadWriteReceiver<LeaderNode> {
        &mut self.receiver
    }
}
