use crate::leader::follower::{LeaderFollowerHandle, LeaderFollowerMessage, LeaderFollowerTask};
use crate::leader::LeaderHandle;
use tokio::sync::mpsc;
use zlambda_common::message::{
    FollowerToLeaderMessageStreamReader, LeaderToFollowerMessageStreamWriter,
};
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollowerBuilder {
    sender: mpsc::Sender<LeaderFollowerMessage>,
    receiver: mpsc::Receiver<LeaderFollowerMessage>,
}

impl LeaderFollowerBuilder {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(16);

        Self { sender, receiver }
    }

    pub fn handle(&self) -> LeaderFollowerHandle {
        LeaderFollowerHandle::new(self.sender.clone())
    }

    pub fn build(
        self,
        id: NodeId,
        reader: Option<FollowerToLeaderMessageStreamReader>,
        writer: Option<LeaderToFollowerMessageStreamWriter>,
        leader_handle: LeaderHandle,
    ) -> LeaderFollowerTask {
        LeaderFollowerTask::new(id, self.receiver, reader, writer, leader_handle)
    }
}
