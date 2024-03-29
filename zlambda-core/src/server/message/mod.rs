mod input;
mod output;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use input::*;
pub use output::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::message::{AsynchronousMessage, SynchronizableMessage, SynchronousMessage};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerSocketAcceptMessage = AsynchronousMessage<ServerSocketAcceptMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerRegistrationMessage =
    SynchronousMessage<ServerRegistrationMessageInput, ServerRegistrationMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerCommitRegistrationMessage =
    SynchronousMessage<ServerCommitRegistrationMessageInput, ServerRegistrationMessageOutput>;

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

pub type ServerLogEntriesRecoveryMessage =
    AsynchronousMessage<ServerLogEntriesRecoveryMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLogEntriesGetMessage =
    SynchronousMessage<ServerLogEntriesGetMessageInput, ServerLogEntriesGetMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLogEntriesCommitMessage = AsynchronousMessage<ServerLogEntriesCommitMessageInput>;

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

pub type ServerClientRegistrationMessage =
    AsynchronousMessage<ServerClientRegistrationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientResignationMessage = AsynchronousMessage<ServerClientResignationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLogAppendRequestMessage = AsynchronousMessage<ServerLogAppendRequestMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLogAppendInitiateMessage =
    SynchronousMessage<ServerLogAppendInitiateMessageInput, ServerLogAppendInitiateMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerServerSocketAddressGetMessage = SynchronousMessage<
    ServerServerSocketAddressGetMessageInput,
    ServerServerSocketAddressGetMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerServerIdGetMessage =
    SynchronousMessage<ServerServerIdGetMessageInput, ServerServerIdGetMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLeaderServerIdGetMessage =
    SynchronousMessage<ServerLeaderServerIdGetMessageInput, ServerLeaderServerIdGetMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerServerNodeMessageSenderGetMessage = SynchronousMessage<
    ServerServerNodeMessageSenderGetMessageInput,
    ServerServerNodeMessageSenderGetMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerServerNodeMessageSenderGetAllMessage = SynchronousMessage<
    ServerServerNodeMessageSenderGetAllMessageInput,
    ServerServerNodeMessageSenderGetAllMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerCommitMessage = SynchronizableMessage<ServerCommitMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerCommitCommitMessage = AsynchronousMessage<ServerCommitCommitMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerServerSocketAddressesGetMessage = SynchronousMessage<
    ServerServerSocketAddressesGetMessageInput,
    ServerServerSocketAddressesGetMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerLogCreateMessage =
    SynchronousMessage<ServerLogCreateMessageInput, ServerLogCreateMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerCommitLogCreateMessage = AsynchronousMessage<ServerCommitLogCreateMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientGetMessage =
    SynchronousMessage<ServerClientGetMessageInput, ServerClientGetMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerConnectMessage = AsynchronousMessage<ServerConnectMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerDisconnectMessage = AsynchronousMessage<ServerDisconnectMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerMessage {
    Ping,
    SocketAccept(ServerSocketAcceptMessage),
    Registration(ServerRegistrationMessage),
    CommitRegistration(ServerCommitRegistrationMessage),
    Recovery(ServerRecoveryMessage),
    LogEntriesReplication(ServerLogEntriesReplicationMessage),
    LogEntriesAcknowledgement(ServerLogEntriesAcknowledgementMessage),
    LogEntriesRecovery(ServerLogEntriesRecoveryMessage),
    LogEntriesGet(ServerLogEntriesGetMessage),
    LogEntriesCommit(ServerLogEntriesCommitMessage),
    LogAppendRequest(ServerLogAppendRequestMessage),
    LogAppendInitiate(ServerLogAppendInitiateMessage),
    ModuleGet(ServerModuleGetMessage),
    ModuleLoad(ServerModuleLoadMessage),
    ModuleUnload(ServerModuleUnloadMessage),
    ClientRegistration(ServerClientRegistrationMessage),
    ClientResignation(ServerClientResignationMessage),
    ServerSocketAddressGet(ServerServerSocketAddressGetMessage),
    ServerIdGet(ServerServerIdGetMessage),
    LeaderServerIdGet(ServerLeaderServerIdGetMessage),
    ServerNodeMessageSenderGet(ServerServerNodeMessageSenderGetMessage),
    ServerNodeMessageSenderGetAll(ServerServerNodeMessageSenderGetAllMessage),
    Commit(ServerCommitMessage),
    CommitCommit(ServerCommitCommitMessage),
    ServerSocketAddressesGet(ServerServerSocketAddressesGetMessage),
    LogCreate(ServerLogCreateMessage),
    CommitLogCreate(ServerCommitLogCreateMessage),
    ServerClientGet(ServerClientGetMessage),
    ServerConnect(ServerConnectMessage),
    ServerDisconnect(ServerDisconnectMessage),
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

impl From<ServerLogEntriesGetMessage> for ServerMessage {
    fn from(message: ServerLogEntriesGetMessage) -> Self {
        Self::LogEntriesGet(message)
    }
}

impl From<ServerLogEntriesCommitMessage> for ServerMessage {
    fn from(message: ServerLogEntriesCommitMessage) -> Self {
        Self::LogEntriesCommit(message)
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

impl From<ServerClientRegistrationMessage> for ServerMessage {
    fn from(message: ServerClientRegistrationMessage) -> Self {
        Self::ClientRegistration(message)
    }
}

impl From<ServerClientResignationMessage> for ServerMessage {
    fn from(message: ServerClientResignationMessage) -> Self {
        Self::ClientResignation(message)
    }
}

impl From<ServerLogAppendRequestMessage> for ServerMessage {
    fn from(message: ServerLogAppendRequestMessage) -> Self {
        Self::LogAppendRequest(message)
    }
}

impl From<ServerLogAppendInitiateMessage> for ServerMessage {
    fn from(message: ServerLogAppendInitiateMessage) -> Self {
        Self::LogAppendInitiate(message)
    }
}

impl From<ServerServerSocketAddressGetMessage> for ServerMessage {
    fn from(message: ServerServerSocketAddressGetMessage) -> Self {
        Self::ServerSocketAddressGet(message)
    }
}

impl From<ServerServerIdGetMessage> for ServerMessage {
    fn from(message: ServerServerIdGetMessage) -> Self {
        Self::ServerIdGet(message)
    }
}

impl From<ServerLeaderServerIdGetMessage> for ServerMessage {
    fn from(message: ServerLeaderServerIdGetMessage) -> Self {
        Self::LeaderServerIdGet(message)
    }
}

impl From<ServerServerNodeMessageSenderGetMessage> for ServerMessage {
    fn from(message: ServerServerNodeMessageSenderGetMessage) -> Self {
        Self::ServerNodeMessageSenderGet(message)
    }
}

impl From<ServerServerNodeMessageSenderGetAllMessage> for ServerMessage {
    fn from(message: ServerServerNodeMessageSenderGetAllMessage) -> Self {
        Self::ServerNodeMessageSenderGetAll(message)
    }
}

impl From<ServerCommitMessage> for ServerMessage {
    fn from(message: ServerCommitMessage) -> Self {
        Self::Commit(message)
    }
}

impl From<ServerCommitCommitMessage> for ServerMessage {
    fn from(message: ServerCommitCommitMessage) -> Self {
        Self::CommitCommit(message)
    }
}

impl From<ServerServerSocketAddressesGetMessage> for ServerMessage {
    fn from(message: ServerServerSocketAddressesGetMessage) -> Self {
        Self::ServerSocketAddressesGet(message)
    }
}

impl From<ServerLogCreateMessage> for ServerMessage {
    fn from(message: ServerLogCreateMessage) -> Self {
        Self::LogCreate(message)
    }
}

impl From<ServerCommitLogCreateMessage> for ServerMessage {
    fn from(message: ServerCommitLogCreateMessage) -> Self {
        Self::CommitLogCreate(message)
    }
}

impl From<ServerClientGetMessage> for ServerMessage {
    fn from(message: ServerClientGetMessage) -> Self {
        Self::ServerClientGet(message)
    }
}

impl From<ServerConnectMessage> for ServerMessage {
    fn from(message: ServerConnectMessage) -> Self {
        Self::ServerConnect(message)
    }
}

impl From<ServerDisconnectMessage> for ServerMessage {
    fn from(message: ServerDisconnectMessage) -> Self {
        Self::ServerDisconnect(message)
    }
}
