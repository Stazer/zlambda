use crate::message::AsynchronousMessage;
use crate::server::member::{ServerMemberReplicationMessageInput, ServerMemberRegistrationMessageInput};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerMemberReplicationMessage = AsynchronousMessage<ServerMemberReplicationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerMemberRegistrationMessage = AsynchronousMessage<ServerMemberRegistrationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerMemberMessage {
    Replication(ServerMemberReplicationMessage),
    Registration(ServerMemberRegistrationMessage),
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
