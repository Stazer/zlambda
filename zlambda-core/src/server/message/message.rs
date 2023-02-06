use crate::common::message::{AsynchronousMessage, SynchronousMessage};
use crate::server::{
    ServerCommitRegistrationMessageInput, ServerLeaderGeneralMessageMessageInput,
    ServerLogEntriesAcknowledgementMessageInput, ServerLogEntriesRecoveryMessageInput,
    ServerLogEntriesReplicationMessageInput, ServerLogEntriesReplicationMessageOutput,
    ServerModuleGetMessageInput, ServerModuleGetMessageOutput, ServerModuleLoadMessageInput,
    ServerModuleLoadMessageOutput, ServerModuleUnloadMessageInput, ServerModuleUnloadMessageOutput,
    ServerNotifyMessageInput, ServerRecoveryMessageInput, ServerRecoveryMessageOutput,
    ServerRegistrationMessageInput, ServerRegistrationMessageOutput,
    ServerSocketAcceptMessageInput,
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

pub type ServerModuleGetMessage =
    SynchronousMessage<ServerModuleGetMessageInput, ServerModuleGetMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerModuleLoadMessage =
    SynchronousMessage<ServerModuleLoadMessageInput, ServerModuleLoadMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerModuleUnloadMessage =
    SynchronousMessage<ServerModuleUnloadMessageInput, ServerModuleUnloadMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNotifyMessage = AsynchronousMessage<ServerNotifyMessageInput>;

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
    ModuleGet(ServerModuleGetMessage),
    ModuleLoad(ServerModuleLoadMessage),
    ModuleUnload(ServerModuleUnloadMessage),
    Notify(ServerNotifyMessage),
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

impl From<ServerModuleGetMessage> for ServerMessage {
    fn from(message: ServerModuleGetMessage) -> Self {
        Self::ModuleGet(message)
    }
}

impl From<ServerModuleLoadMessage> for ServerMessage {
    fn from(message: ServerModuleLoadMessage) -> Self {
        Self::ModuleLoad(message)
    }
}

impl From<ServerModuleUnloadMessage> for ServerMessage {
    fn from(message: ServerModuleUnloadMessage) -> Self {
        Self::ModuleUnload(message)
    }
}

impl From<ServerNotifyMessage> for ServerMessage {
    fn from(message: ServerNotifyMessage) -> Self {
        Self::Notify(message)
    }
}
