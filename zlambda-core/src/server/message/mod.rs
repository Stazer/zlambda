use crate::message::{AsynchronousMessage, SynchronousMessage};
use crate::server::{
    ServerCommitRegistrationMessageInput, ServerLeaderGeneralMessageMessageInput,
    ServerLogEntriesAcknowledgementMessageInput, ServerLogEntriesRecoveryMessageInput,
    ServerLogEntriesReplicationMessageInput, ServerLogEntriesReplicationMessageOutput,
    ServerRecoveryMessageInput, ServerRecoveryMessageOutput, ServerRegistrationMessageInput,
    ServerRegistrationMessageOutput, ServerSocketAcceptMessageInput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerSocketAcceptMessage = AsynchronousMessage<ServerSocketAcceptMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLeaderGeneralMessageMessage =
    AsynchronousMessage<ServerLeaderGeneralMessageMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerRegistrationMessage =
    SynchronousMessage<ServerRegistrationMessageInput, ServerRegistrationMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerRecoveryMessage =
    SynchronousMessage<ServerRecoveryMessageInput, ServerRecoveryMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLogEntriesReplicationMessage = SynchronousMessage<
    ServerLogEntriesReplicationMessageInput,
    ServerLogEntriesReplicationMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLogEntriesAcknowledgementMessage =
    AsynchronousMessage<ServerLogEntriesAcknowledgementMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerCommitRegistrationMessage =
    SynchronousMessage<ServerCommitRegistrationMessageInput, ServerRegistrationMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLogEntriesRecoveryMessage =
    AsynchronousMessage<ServerLogEntriesRecoveryMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerMessage {
    Ping,
    SocketAccept(ServerSocketAcceptMessage),
    LeaderGeneralMessage(ServerLeaderGeneralMessageMessage),
    Registration(ServerRegistrationMessage),
    CommitRegistration(ServerCommitRegistrationMessage),
    Recovery(ServerRecoveryMessage),
    LogEntriesReplication(ServerLogEntriesReplicationMessage),
    LogEntriesAcknowledgement(ServerLogEntriesAcknowledgementMessage),
    LogEntriesRecovery(ServerLogEntriesRecoveryMessage),
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

impl From<ServerCommitRegistrationMessage> for ServerMessage {
    fn from(message: ServerCommitRegistrationMessage) -> Self {
        Self::CommitRegistration(message)
    }
}

impl From<ServerRecoveryMessage> for ServerMessage {
    fn from(message: ServerRecoveryMessage) -> Self {
        Self::Recovery(message)
    }
}

impl From<ServerLogEntriesReplicationMessage> for ServerMessage {
    fn from(message: ServerLogEntriesReplicationMessage) -> Self {
        Self::LogEntriesReplication(message)
    }
}

impl From<ServerLogEntriesAcknowledgementMessage> for ServerMessage {
    fn from(message: ServerLogEntriesAcknowledgementMessage) -> Self {
        Self::LogEntriesAcknowledgement(message)
    }
}

impl From<ServerLogEntriesRecoveryMessage> for ServerMessage {
    fn from(message: ServerLogEntriesRecoveryMessage) -> Self {
        Self::LogEntriesRecovery(message)
    }
}
