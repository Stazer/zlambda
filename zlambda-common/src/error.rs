use std::error;
use std::fmt::{self, Debug, Display, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SimpleError {
    message: String,
}

impl Debug for SimpleError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{}", self.message)
    }
}

impl Display for SimpleError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{}", self.message)
    }
}

impl error::Error for SimpleError {}

impl SimpleError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}
