use crate::common::message::AsynchronousMessage;
use crate::server::node::{
    ServerNodeNotifyMessageInput, ServerNodeRecoveryMessageInput,
    ServerNodeRegistrationMessageInput, ServerNodeReplicationMessageInput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeReplicationMessage = AsynchronousMessage<ServerNodeReplicationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeRegistrationMessage = AsynchronousMessage<ServerNodeRegistrationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeRecoveryMessage = AsynchronousMessage<ServerNodeRecoveryMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeNotifyMessage = AsynchronousMessage<ServerNodeNotifyMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerNodeMessage {
    Replication(ServerNodeReplicationMessage),
    Registration(ServerNodeRegistrationMessage),
    Recovery(ServerNodeRecoveryMessage),
    Notify(ServerNodeNotifyMessage),
}

impl From<ServerNodeReplicationMessage> for ServerNodeMessage {
    fn from(message: ServerNodeReplicationMessage) -> Self {
        Self::Replication(message)
    }
}

impl From<ServerNodeRegistrationMessage> for ServerNodeMessage {
    fn from(message: ServerNodeRegistrationMessage) -> Self {
        Self::Registration(message)
    }
}

impl From<ServerNodeRecoveryMessage> for ServerNodeMessage {
    fn from(message: ServerNodeRecoveryMessage) -> Self {
        Self::Recovery(message)
    }
}

impl From<ServerNodeNotifyMessage> for ServerNodeMessage {
    fn from(message: ServerNodeNotifyMessage) -> Self {
        Self::Notify(message)
    }
}
