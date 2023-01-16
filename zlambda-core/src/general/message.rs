use crate::general::{
    GeneralRecoveryRequestMessageInput, GeneralRecoveryResponseMessageInput,
    GeneralRegistrationRequestMessageInput, GeneralRegistrationResponseMessageInput,
};
use crate::message::AsynchronousMessage;
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralMessage {
    RegistrationRequest(GeneralRegistrationRequestMessage),
    RegistrationResponse(GeneralRegistrationResponseMessage),
    RecoveryRequest(GeneralRecoveryRequestMessage),
    RecoveryResponse(GeneralRecoveryResponseMessage),
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
