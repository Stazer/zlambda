use crate::common::message::AsynchronousMessage;
use crate::general::{
    GeneralClientRedirectMessageInput, GeneralClientRegistrationRequestMessageInput,
    GeneralClientRegistrationResponseMessageInput, GeneralLogEntriesAppendInitiateMessageInput,
    GeneralLogEntriesAppendRequestMessageInput, GeneralLogEntriesAppendResponseMessageInput,
    GeneralLogEntriesCommitMessageInput, GeneralNodeHandshakeRequestMessageInput,
    GeneralNodeHandshakeResponseMessageInput, GeneralNotificationMessageInput,
    GeneralRecoveryRequestMessageInput, GeneralRecoveryResponseMessageInput,
    GeneralRegistrationRequestMessageInput, GeneralRegistrationResponseMessageInput,
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

pub type GeneralNodeHandshakeRequestMessage =
    AsynchronousMessage<GeneralNodeHandshakeRequestMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralNodeHandshakeResponseMessage =
    AsynchronousMessage<GeneralNodeHandshakeResponseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralLogEntriesAppendRequestMessage =
    AsynchronousMessage<GeneralLogEntriesAppendRequestMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralLogEntriesAppendResponseMessage =
    AsynchronousMessage<GeneralLogEntriesAppendResponseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralLogEntriesAppendInitiateMessage =
    AsynchronousMessage<GeneralLogEntriesAppendInitiateMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralLogEntriesCommitMessage = AsynchronousMessage<GeneralLogEntriesCommitMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralClientRegistrationRequestMessage =
    AsynchronousMessage<GeneralClientRegistrationRequestMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralClientRegistrationResponseMessage =
    AsynchronousMessage<GeneralClientRegistrationResponseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralNotificationMessage = AsynchronousMessage<GeneralNotificationMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GeneralClientRedirectMessage = AsynchronousMessage<GeneralClientRedirectMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralMessage {
    RegistrationRequest(GeneralRegistrationRequestMessage),
    RegistrationResponse(GeneralRegistrationResponseMessage),
    RecoveryRequest(GeneralRecoveryRequestMessage),
    RecoveryResponse(GeneralRecoveryResponseMessage),
    NodeHandshakeRequest(GeneralNodeHandshakeRequestMessage),
    NodeHandshakeResponse(GeneralNodeHandshakeResponseMessage),
    LogEntriesAppendRequest(GeneralLogEntriesAppendRequestMessage),
    LogEntriesAppendResponse(GeneralLogEntriesAppendResponseMessage),
    LogEntriesAppendInitiate(GeneralLogEntriesAppendInitiateMessage),
    LogEntriesCommit(GeneralLogEntriesCommitMessage),
    ClientRegistrationRequest(GeneralClientRegistrationRequestMessage),
    ClientRegistrationResponse(GeneralClientRegistrationResponseMessage),
    Notification(GeneralNotificationMessage),
    ClientRedirect(GeneralClientRedirectMessage),
}

impl From<GeneralRegistrationRequestMessage> for GeneralMessage {
    fn from(message: GeneralRegistrationRequestMessage) -> Self {
        Self::RegistrationRequest(message)
    }
}

impl From<GeneralRegistrationResponseMessage> for GeneralMessage {
    fn from(message: GeneralRegistrationResponseMessage) -> Self {
        Self::RegistrationResponse(message)
    }
}

impl From<GeneralRecoveryRequestMessage> for GeneralMessage {
    fn from(message: GeneralRecoveryRequestMessage) -> Self {
        Self::RecoveryRequest(message)
    }
}

impl From<GeneralRecoveryResponseMessage> for GeneralMessage {
    fn from(message: GeneralRecoveryResponseMessage) -> Self {
        Self::RecoveryResponse(message)
    }
}

impl From<GeneralNodeHandshakeRequestMessage> for GeneralMessage {
    fn from(message: GeneralNodeHandshakeRequestMessage) -> Self {
        Self::NodeHandshakeRequest(message)
    }
}

impl From<GeneralNodeHandshakeResponseMessage> for GeneralMessage {
    fn from(message: GeneralNodeHandshakeResponseMessage) -> Self {
        Self::NodeHandshakeResponse(message)
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

impl From<GeneralLogEntriesAppendInitiateMessage> for GeneralMessage {
    fn from(message: GeneralLogEntriesAppendInitiateMessage) -> Self {
        Self::LogEntriesAppendInitiate(message)
    }
}

impl From<GeneralLogEntriesCommitMessage> for GeneralMessage {
    fn from(message: GeneralLogEntriesCommitMessage) -> Self {
        Self::LogEntriesCommit(message)
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

impl From<GeneralClientRedirectMessage> for GeneralMessage {
    fn from(message: GeneralClientRedirectMessage) -> Self {
        Self::ClientRedirect(message)
    }
}
