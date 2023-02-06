use crate::common::message::AsynchronousMessage;
use crate::general::{
    GeneralLogEntriesAppendRequestMessageInput, GeneralLogEntriesAppendResponseMessageInput,
    GeneralNotifyMessageInput, GeneralRecoveryRequestMessageInput,
    GeneralRecoveryResponseMessageInput, GeneralRegistrationRequestMessageInput,
    GeneralRegistrationResponseMessageInput,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralMessage {
    RegistrationRequest(GeneralRegistrationRequestMessage),
    RegistrationResponse(GeneralRegistrationResponseMessage),
    RecoveryRequest(GeneralRecoveryRequestMessage),
    RecoveryResponse(GeneralRecoveryResponseMessage),
    LogEntriesAppendRequest(GeneralLogEntriesAppendRequestMessage),
    LogEntriesAppendResponse(GeneralLogEntriesAppendResponseMessage),
    Notify(GeneralNotifyMessage),
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
