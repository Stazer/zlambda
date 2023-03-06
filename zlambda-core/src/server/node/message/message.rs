use crate::common::message::{AsynchronousMessage, SynchronousMessage};
use crate::server::node::{
    ServerNodeLogAppendInitiateMessageInput, ServerNodeLogAppendResponseMessageInput,
    ServerNodeNodeHandshakeMessageInput, ServerNodeNotificationEndMessageInput,
    ServerNodeNotificationImmediateMessageInput, ServerNodeNotificationNextMessageInput,
    ServerNodeNotificationStartMessageInput, ServerNodeNotificationStartMessageOutput,
    ServerNodeRecoveryMessageInput, ServerNodeRegistrationMessageInput,
    ServerNodeReplicationMessageInput, ServerNodeShutdownMessageInput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeShutdownMessage = AsynchronousMessage<ServerNodeShutdownMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeReplicationMessage = AsynchronousMessage<ServerNodeReplicationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeRegistrationMessage = AsynchronousMessage<ServerNodeRegistrationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeRecoveryMessage = AsynchronousMessage<ServerNodeRecoveryMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeNodeHandshakeMessage = AsynchronousMessage<ServerNodeNodeHandshakeMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeLogAppendResponseMessage =
    AsynchronousMessage<ServerNodeLogAppendResponseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeLogAppendInitiateMessage =
    AsynchronousMessage<ServerNodeLogAppendInitiateMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeNotificationImmediateMessage =
    AsynchronousMessage<ServerNodeNotificationImmediateMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeNotificationStartMessage = SynchronousMessage<
    ServerNodeNotificationStartMessageInput,
    ServerNodeNotificationStartMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeNotificationNextMessage =
    AsynchronousMessage<ServerNodeNotificationNextMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeNotificationEndMessage =
    AsynchronousMessage<ServerNodeNotificationEndMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerNodeMessage {
    Shutdown(ServerNodeShutdownMessage),
    Replication(ServerNodeReplicationMessage),
    Registration(ServerNodeRegistrationMessage),
    Recovery(ServerNodeRecoveryMessage),
    NodeHandshake(ServerNodeNodeHandshakeMessage),
    LogAppendResponse(ServerNodeLogAppendResponseMessage),
    LogAppendInitiate(ServerNodeLogAppendInitiateMessage),
    NotificationImmediate(ServerNodeNotificationImmediateMessage),
    NotificationStart(ServerNodeNotificationStartMessage),
    NotificationNext(ServerNodeNotificationNextMessage),
    NotificationEnd(ServerNodeNotificationEndMessage),
}

impl From<ServerNodeShutdownMessage> for ServerNodeMessage {
    fn from(message: ServerNodeShutdownMessage) -> Self {
        Self::Shutdown(message)
    }
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

impl From<ServerNodeNodeHandshakeMessage> for ServerNodeMessage {
    fn from(message: ServerNodeNodeHandshakeMessage) -> Self {
        Self::NodeHandshake(message)
    }
}

impl From<ServerNodeLogAppendResponseMessage> for ServerNodeMessage {
    fn from(message: ServerNodeLogAppendResponseMessage) -> Self {
        Self::LogAppendResponse(message)
    }
}

impl From<ServerNodeLogAppendInitiateMessage> for ServerNodeMessage {
    fn from(message: ServerNodeLogAppendInitiateMessage) -> Self {
        Self::LogAppendInitiate(message)
    }
}

impl From<ServerNodeNotificationImmediateMessage> for ServerNodeMessage {
    fn from(message: ServerNodeNotificationImmediateMessage) -> Self {
        Self::NotificationImmediate(message)
    }
}

impl From<ServerNodeNotificationStartMessage> for ServerNodeMessage {
    fn from(message: ServerNodeNotificationStartMessage) -> Self {
        Self::NotificationStart(message)
    }
}

impl From<ServerNodeNotificationNextMessage> for ServerNodeMessage {
    fn from(message: ServerNodeNotificationNextMessage) -> Self {
        Self::NotificationNext(message)
    }
}

impl From<ServerNodeNotificationEndMessage> for ServerNodeMessage {
    fn from(message: ServerNodeNotificationEndMessage) -> Self {
        Self::NotificationEnd(message)
    }
}
