use bytes::{BufMut, BytesMut};
use postcard::serialize_with_flavor;
use serde::Serialize;
use serde_json::to_vec;
use std::error;
use std::fmt::{self, Debug, Display, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

struct BytesMutFlavor(BytesMut);

impl postcard::ser_flavors::Flavor for BytesMutFlavor {
    type Output = BytesMut;

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        self.0.put_u8(data);

        Ok(())
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self.0)
    }

    fn try_extend(&mut self, data: &[u8]) -> postcard::Result<()> {
        self.0.extend_from_slice(data);

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SerializeError(postcard::Error);

impl Debug for SerializeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, formatter)
    }
}

impl Display for SerializeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

impl error::Error for SerializeError {}

impl From<postcard::Error> for SerializeError {
    fn from(error: postcard::Error) -> Self {
        Self::new(error)
    }
}

impl SerializeError {
    pub fn new(error: postcard::Error) -> Self {
        Self(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn serialize_to_bytes<T>(value: &T) -> Result<BytesMut, SerializeError>
where
    T: Serialize,
{
    Ok(serialize_with_flavor(
        value,
        BytesMutFlavor(BytesMut::new()),
    )?)
}
