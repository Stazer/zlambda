mod input;
mod output;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use input::*;
pub use output::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::message::{AsynchronousMessage, SynchronousMessage};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientShutdownMessage = AsynchronousMessage<ServerClientShutdownMessageInput>;

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
    Shutdown(ServerClientShutdownMessage),
    NotificationImmediate(ServerClientNotificationImmediateMessage),
    NotificationStart(ServerClientNotificationStartMessage),
    NotificationNext(ServerClientNotificationNextMessage),
    NotificationEnd(ServerClientNotificationEndMessage),
}

impl From<ServerClientShutdownMessage> for ServerClientMessage {
    fn from(message: ServerClientShutdownMessage) -> Self {
        Self::Shutdown(message)
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
