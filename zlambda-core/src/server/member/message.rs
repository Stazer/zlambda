use crate::message::AsynchronousMessage;
use crate::server::member::{
    ServerMemberRecoveryMessageInput, ServerMemberRegistrationMessageInput,
    ServerMemberReplicationMessageInput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerMemberReplicationMessage = AsynchronousMessage<ServerMemberReplicationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerMemberRegistrationMessage =
    AsynchronousMessage<ServerMemberRegistrationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerMemberRecoveryMessage = AsynchronousMessage<ServerMemberRecoveryMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerMemberMessage {
    Replication(ServerMemberReplicationMessage),
    Registration(ServerMemberRegistrationMessage),
    Recovery(ServerMemberRecoveryMessage),
}

impl From<ServerMemberReplicationMessage> for ServerMemberMessage {
    fn from(message: ServerMemberReplicationMessage) -> Self {
        Self::Replication(message)
    }
}

impl From<ServerMemberRegistrationMessage> for ServerMemberMessage {
    fn from(message: ServerMemberRegistrationMessage) -> Self {
        Self::Registration(message)
    }
}

impl From<ServerMemberRecoveryMessage> for ServerMemberMessage {
    fn from(message: ServerMemberRecoveryMessage) -> Self {
        Self::Recovery(message)
    }
}
