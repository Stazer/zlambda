use crate::common::message::{AsynchronousMessage, SynchronousMessage};
use crate::server::client::{
    ServerClientNotificationEndMessageInput, ServerClientNotificationImmediateMessageInput,
    ServerClientNotificationNextMessageInput, ServerClientNotificationStartMessageInput,
    ServerClientNotificationStartMessageOutput, ServerClientNotifyMessageInput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientNotifyMessage = AsynchronousMessage<ServerClientNotifyMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientNotificationImmediateMessage =
    AsynchronousMessage<ServerClientNotificationImmediateMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientNotificationStartMessage = SynchronousMessage<
    ServerClientNotificationStartMessageInput,
    ServerClientNotificationStartMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientNotificationNextMessage =
    AsynchronousMessage<ServerClientNotificationNextMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientNotificationEndMessage =
    AsynchronousMessage<ServerClientNotificationEndMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerClientMessage {
    Notify(ServerClientNotifyMessage),
    NotificationImmediate(ServerClientNotificationImmediateMessage),
    NotificationStart(ServerClientNotificationStartMessage),
    NotificationNext(ServerClientNotificationNextMessage),
    NotificationEnd(ServerClientNotificationEndMessage),
}

impl From<ServerClientNotifyMessage> for ServerClientMessage {
    fn from(message: ServerClientNotifyMessage) -> Self {
        Self::Notify(message)
    }
}

impl From<ServerClientNotificationImmediateMessage> for ServerClientMessage {
    fn from(message: ServerClientNotificationImmediateMessage) -> Self {
        Self::NotificationImmediate(message)
    }
}

impl From<ServerClientNotificationStartMessage> for ServerClientMessage {
    fn from(message: ServerClientNotificationStartMessage) -> Self {
        Self::NotificationStart(message)
    }
}

impl From<ServerClientNotificationNextMessage> for ServerClientMessage {
    fn from(message: ServerClientNotificationNextMessage) -> Self {
        Self::NotificationNext(message)
    }
}

impl From<ServerClientNotificationEndMessage> for ServerClientMessage {
    fn from(message: ServerClientNotificationEndMessage) -> Self {
        Self::NotificationEnd(message)
    }
}
