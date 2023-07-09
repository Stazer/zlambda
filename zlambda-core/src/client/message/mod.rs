mod input;
mod output;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use input::*;
pub use output::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::message::{AsynchronousMessage, SynchronousMessage};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientNotificationImmediateMessage =
    AsynchronousMessage<ClientNotificationImmediateMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientNotificationStartMessage =
    SynchronousMessage<ClientNotificationStartMessageInput, ClientNotificationStartMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientNotificationNextMessage = AsynchronousMessage<ClientNotificationNextMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientNotificationEndMessage = AsynchronousMessage<ClientNotificationEndMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientExitMessage = AsynchronousMessage<ClientExitMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClientMessage {
    NotificationImmediate(ClientNotificationImmediateMessage),
    NotificationStart(ClientNotificationStartMessage),
    NotificationNext(ClientNotificationNextMessage),
    NotificationEnd(ClientNotificationEndMessage),
    Exit(ClientExitMessage),
}

impl From<ClientNotificationImmediateMessage> for ClientMessage {
    fn from(message: ClientNotificationImmediateMessage) -> Self {
        Self::NotificationImmediate(message)
    }
}

impl From<ClientNotificationStartMessage> for ClientMessage {
    fn from(message: ClientNotificationStartMessage) -> Self {
        Self::NotificationStart(message)
    }
}

impl From<ClientNotificationNextMessage> for ClientMessage {
    fn from(message: ClientNotificationNextMessage) -> Self {
        Self::NotificationNext(message)
    }
}

impl From<ClientNotificationEndMessage> for ClientMessage {
    fn from(message: ClientNotificationEndMessage) -> Self {
        Self::NotificationEnd(message)
    }
}

impl From<ClientExitMessage> for ClientMessage {
    fn from(message: ClientExitMessage) -> Self {
        Self::Exit(message)
    }
}
