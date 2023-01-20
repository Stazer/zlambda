use crate::message::{MessageStreamReader, MessageStreamWriter};
use std::error::Error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeMemberAction {
    Continue,
    Stop,
    Error(Box<dyn Error + Send>),
}