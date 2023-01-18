use std::error;
use std::fmt::{self, Display, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LogError {
    NotExisting,
    NotAcknowledgeable,
    AlreadyAcknowledged,
}

impl Display for LogError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotExisting => write!(formatter, "not existing"),
            Self::NotAcknowledgeable => write!(formatter, "not acknowledgeable"),
            Self::AlreadyAcknowledged => write!(formatter, "already acknowledged"),
        }
    }
}

impl error::Error for LogError {}
