use crate::common::message::AsynchronousMessage;
use crate::general::{
    GeneralClientRegistrationRequestMessageInput, GeneralClientRegistrationResponseMessageInput,
    GeneralLogEntriesAppendRequestMessageInput, GeneralLogEntriesAppendResponseMessageInput,
    GeneralNotifyMessageInput, GeneralRecoveryRequestMessageInput,
    GeneralRecoveryResponseMessageInput, GeneralRegistrationRequestMessageInput,
    GeneralRegistrationResponseMessageInput, GeneralNotificationMessageInput,
};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralRegistrationRequestMessage =
    AsynchronousMessage<GeneralRegistrationRequestMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralRegistrationResponseMessage =
    AsynchronousMessage<GeneralRegistrationResponseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralRecoveryRequestMessage = AsynchronousMessage<GeneralRecoveryRequestMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralRecoveryResponseMessage = AsynchronousMessage<GeneralRecoveryResponseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralLogEntriesAppendRequestMessage =
    AsynchronousMessage<GeneralLogEntriesAppendRequestMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralLogEntriesAppendResponseMessage =
    AsynchronousMessage<GeneralLogEntriesAppendResponseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralNotifyMessage = AsynchronousMessage<GeneralNotifyMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralClientRegistrationRequestMessage =
    AsynchronousMessage<GeneralClientRegistrationRequestMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralClientRegistrationResponseMessage =
    AsynchronousMessage<GeneralClientRegistrationResponseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralNotificationMessage = AsynchronousMessage<GeneralNotificationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralMessage {
    RegistrationRequest(GeneralRegistrationRequestMessage),
    RegistrationResponse(GeneralRegistrationResponseMessage),
    RecoveryRequest(GeneralRecoveryRequestMessage),
    RecoveryResponse(GeneralRecoveryResponseMessage),
    LogEntriesAppendRequest(GeneralLogEntriesAppendRequestMessage),
    LogEntriesAppendResponse(GeneralLogEntriesAppendResponseMessage),
    Notify(GeneralNotifyMessage),
    ClientRegistrationRequest(GeneralClientRegistrationRequestMessage),
    ClientRegistrationResponse(GeneralClientRegistrationResponseMessage),
    Notification(GeneralNotificationMessage),
}

impl From<GeneralRegistrationRequestMessage> for GeneralMessage {
    fn from(message: GeneralRegistrationRequestMessage) -> Self {
        GeneralMessage::RegistrationRequest(message)
    }
}

impl From<GeneralRegistrationResponseMessage> for GeneralMessage {
    fn from(message: GeneralRegistrationResponseMessage) -> Self {
        GeneralMessage::RegistrationResponse(message)
    }
}

impl From<GeneralRecoveryRequestMessage> for GeneralMessage {
    fn from(message: GeneralRecoveryRequestMessage) -> Self {
        GeneralMessage::RecoveryRequest(message)
    }
}

impl From<GeneralRecoveryResponseMessage> for GeneralMessage {
    fn from(message: GeneralRecoveryResponseMessage) -> Self {
        GeneralMessage::RecoveryResponse(message)
    }
}

impl From<GeneralLogEntriesAppendRequestMessage> for GeneralMessage {
    fn from(message: GeneralLogEntriesAppendRequestMessage) -> Self {
        Self::LogEntriesAppendRequest(message)
    }
}

impl From<GeneralLogEntriesAppendResponseMessage> for GeneralMessage {
    fn from(message: GeneralLogEntriesAppendResponseMessage) -> Self {
        Self::LogEntriesAppendResponse(message)
    }
}

impl From<GeneralNotifyMessage> for GeneralMessage {
    fn from(message: GeneralNotifyMessage) -> Self {
        Self::Notify(message)
    }
}

impl From<GeneralClientRegistrationRequestMessage> for GeneralMessage {
    fn from(message: GeneralClientRegistrationRequestMessage) -> Self {
        Self::ClientRegistrationRequest(message)
    }
}

impl From<GeneralClientRegistrationResponseMessage> for GeneralMessage {
    fn from(message: GeneralClientRegistrationResponseMessage) -> Self {
        Self::ClientRegistrationResponse(message)
    }
}

impl From<GeneralNotificationMessage> for GeneralMessage {
    fn from(message: GeneralNotificationMessage) -> Self {
        Self::Notification(message)
    }
}
