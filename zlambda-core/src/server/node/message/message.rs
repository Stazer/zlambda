use crate::common::message::{AsynchronousMessage, SynchronousMessage};
use crate::server::node::{
    ServerNodeLogAppendResponseMessageInput, ServerNodeNotificationEndMessageInput,
    ServerNodeNotificationImmediateMessageInput, ServerNodeNotificationNextMessageInput,
    ServerNodeNotificationStartMessageInput, ServerNodeNotificationStartMessageOutput,
    ServerNodeRecoveryMessageInput,
    ServerNodeRegistrationMessageInput, ServerNodeReplicationMessageInput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeReplicationMessage = AsynchronousMessage<ServerNodeReplicationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeRegistrationMessage = AsynchronousMessage<ServerNodeRegistrationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeRecoveryMessage = AsynchronousMessage<ServerNodeRecoveryMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerNodeLogAppendResponseMessage =
    AsynchronousMessage<ServerNodeLogAppendResponseMessageInput>;

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
    Replication(ServerNodeReplicationMessage),
    Registration(ServerNodeRegistrationMessage),
    Recovery(ServerNodeRecoveryMessage),
    LogAppendResponse(ServerNodeLogAppendResponseMessage),
    NotificationImmediate(ServerNodeNotificationImmediateMessage),
    NotificationStart(ServerNodeNotificationStartMessage),
    NotificationNext(ServerNodeNotificationNextMessage),
    NotificationEnd(ServerNodeNotificationEndMessage),
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

impl From<ServerNodeLogAppendResponseMessage> for ServerNodeMessage {
    fn from(message: ServerNodeLogAppendResponseMessage) -> Self {
        Self::LogAppendResponse(message)
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
