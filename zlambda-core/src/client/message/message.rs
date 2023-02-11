use crate::common::message::{AsynchronousMessage, SynchronousMessage};
use crate::client::{ClientNotificationStartMessageInput, ClientNotificationNextMessageInput, ClientNotificationStartMessageOutput};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientNotificationStartMessage = SynchronousMessage<ClientNotificationStartMessageInput, ClientNotificationStartMessageOutput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientNotificationNextMessage = AsynchronousMessage<ClientNotificationNextMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClientMessage {
    NotificationStart(ClientNotificationStartMessage),
    NotificationNext(ClientNotificationNextMessage),
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
