use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GuestToLeaderMessage {}

impl From<Message> for Result<GuestToLeaderMessage, MessageError> {
    fn from(message: Message) -> Self {
        Err(MessageError::UnexpectedMessage(message))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type GuestToLeaderMessageStreamReader = BasicMessageStreamReader<GuestToLeaderMessage>;
pub type GuestToLeaderMessageStreamWriter = BasicMessageStreamWriter<GuestToLeaderMessage>;
