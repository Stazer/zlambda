use bytes::Bytes;
use postcard::take_from_bytes;
use serde::Deserialize;
use std::error;
use std::fmt::{self, Formatter, Display, Debug};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DeserializeError(postcard::Error);

impl Debug for DeserializeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, formatter)
    }
}

impl Display for DeserializeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

impl error::Error for DeserializeError {}

impl From<postcard::Error> for DeserializeError {
    fn from(error: postcard::Error) -> Self {
        Self::new(error)
    }
}

impl DeserializeError {
    pub fn new(error: postcard::Error) -> Self {
        Self(error)
    }
}

pub fn deserialize_from_bytes<'de, T>(bytes: &'de Bytes) -> Result<(T, Bytes), DeserializeError>
where
    T: Deserialize<'de>
{
    let (value, _) = take_from_bytes(bytes)?;

    Ok((value, Bytes::default()))
}
