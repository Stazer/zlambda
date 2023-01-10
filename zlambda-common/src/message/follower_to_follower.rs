use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToFollowerMessage {}

impl From<Message> for Result<FollowerToFollowerMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::FollowerToFollower(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type FollowerToFollowerMessageStreamReader =
    BasicMessageStreamReader<FollowerToFollowerMessage>;
pub type FollowerToFollowerMessageStreamWriter =
    BasicMessageStreamWriter<FollowerToFollowerMessage>;
