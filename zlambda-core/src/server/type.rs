use crate::general::GeneralMessage;
use crate::common::message::{MessageSocketReceiver, MessageSocketSender};
use crate::server::{FollowingLog, LeadingLog, ServerId};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLeaderType {
    log: LeadingLog,
}

impl ServerLeaderType {
    pub fn new(log: LeadingLog) -> Self {
        Self { log }
    }

    pub fn log(&self) -> &LeadingLog {
        &self.log
    }

    pub fn log_mut(&mut self) -> &mut LeadingLog {
        &mut self.log
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerFollowerType {
    leader_server_id: ServerId,
    log: FollowingLog,
    sender: MessageSocketSender<GeneralMessage>,
    receiver: MessageSocketReceiver<GeneralMessage>,
}

impl ServerFollowerType {
    pub fn new(
        leader_server_id: ServerId,
        log: FollowingLog,
        sender: MessageSocketSender<GeneralMessage>,
        receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            leader_server_id,
            log,
            sender,
            receiver,
        }
    }

    pub fn leader_server_id(&self) -> ServerId {
        self.leader_server_id
    }

    pub fn log(&self) -> &FollowingLog {
        &self.log
    }

    pub fn log_mut(&mut self) -> &mut FollowingLog {
        &mut self.log
    }

    pub fn sender(&self) -> &MessageSocketSender<GeneralMessage> {
        &self.sender
    }

    pub fn sender_mut(&mut self) -> &mut MessageSocketSender<GeneralMessage> {
        &mut self.sender
    }

    pub fn receiver(&self) -> &MessageSocketReceiver<GeneralMessage> {
        &self.receiver
    }

    pub fn receiver_mut(&mut self) -> &mut MessageSocketReceiver<GeneralMessage> {
        &mut self.receiver
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerType {
    Leader(ServerLeaderType),
    Follower(ServerFollowerType),
}

impl From<ServerLeaderType> for ServerType {
    fn from(r#type: ServerLeaderType) -> Self {
        Self::Leader(r#type)
    }
}

impl From<ServerFollowerType> for ServerType {
    fn from(r#type: ServerFollowerType) -> Self {
        Self::Follower(r#type)
    }
}
