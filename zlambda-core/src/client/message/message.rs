use crate::client::{
    ClientNotificationEndMessageInput, ClientNotificationImmediateMessageInput,
    ClientNotificationNextMessageInput, ClientNotificationStartMessageInput,
    ClientNotificationStartMessageOutput,
};
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

#[derive(Debug)]
pub enum ClientMessage {
    NotificationImmediate(ClientNotificationImmediateMessage),
    NotificationStart(ClientNotificationStartMessage),
    NotificationNext(ClientNotificationNextMessage),
    NotificationEnd(ClientNotificationEndMessage),
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
