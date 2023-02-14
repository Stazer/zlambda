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
}

impl ServerFollowerType {
    pub fn new(leader_server_id: ServerId, log: FollowingLog) -> Self {
        Self {
            leader_server_id,
            log,
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
