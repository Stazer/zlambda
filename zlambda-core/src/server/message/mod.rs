use crate::message::{AsynchronousMessage, SynchronousMessage};
use crate::server::{
    ServerAcknowledgeLogEntriesMessageInput, ServerRecoveryMessageInput,
    ServerRecoveryMessageOutput, ServerRegistrationMessageInput, ServerRegistrationMessageOutput,
    ServerReplicateLogEntriesMessageInput, ServerReplicateLogEntriesMessageOutput,
    ServerSocketAcceptMessageInput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerSocketAcceptMessage = AsynchronousMessage<ServerSocketAcceptMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerRegistrationMessage =
    SynchronousMessage<ServerRegistrationMessageInput, ServerRegistrationMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerRecoveryMessage =
    SynchronousMessage<ServerRecoveryMessageInput, ServerRecoveryMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerReplicateLogEntriesMessage = SynchronousMessage<
    ServerReplicateLogEntriesMessageInput,
    ServerReplicateLogEntriesMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerAcknowledgeLogEntriesMessage =
    AsynchronousMessage<ServerAcknowledgeLogEntriesMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerMessage {
    SocketAccept(ServerSocketAcceptMessage),
    Registration(ServerRegistrationMessage),
    Recovery(ServerRecoveryMessage),
    ReplicateLogEntries(ServerReplicateLogEntriesMessage),
    AcknowledgeLogEntries(ServerAcknowledgeLogEntriesMessage),
}

impl From<ServerSocketAcceptMessage> for ServerMessage {
    fn from(message: ServerSocketAcceptMessage) -> Self {
        Self::SocketAccept(message)
    }
}

impl From<ServerRegistrationMessage> for ServerMessage {
    fn from(message: ServerRegistrationMessage) -> Self {
        Self::Registration(message)
    }
}

impl From<ServerRecoveryMessage> for ServerMessage {
    fn from(message: ServerRecoveryMessage) -> Self {
        Self::Recovery(message)
    }
}

impl From<ServerReplicateLogEntriesMessage> for ServerMessage {
    fn from(message: ServerReplicateLogEntriesMessage) -> Self {
        Self::ReplicateLogEntries(message)
    }
}

impl From<ServerAcknowledgeLogEntriesMessage> for ServerMessage {
    fn from(message: ServerAcknowledgeLogEntriesMessage) -> Self {
        Self::AcknowledgeLogEntries(message)
    }
}
