use crate::message::{Message, MessageError, BasicMessageStreamReader, BasicMessageStreamWriter};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CandidateToCandidateMessage {}

impl From<Message> for Result<CandidateToCandidateMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::CandidateToCandidate(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type CandidateToCandidateMessageStreamReader =
    BasicMessageStreamReader<CandidateToCandidateMessage>;
pub type CandidateToCandidateMessageStreamWriter =
    BasicMessageStreamWriter<CandidateToCandidateMessage>;
