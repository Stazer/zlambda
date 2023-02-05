use crate::common::message::MessageError;
use bytes::BytesMut;
use postcard::take_from_bytes;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct MessageBufferReader<T>
where
    T: DeserializeOwned,
{
    buffer: BytesMut,
    r#type: PhantomData<T>,
}

impl<T> Default for MessageBufferReader<T>
where
    T: DeserializeOwned,
{
    fn default() -> Self {
        Self {
            buffer: BytesMut::default(),
            r#type: PhantomData::<T>,
        }
    }
}

impl<T> MessageBufferReader<T>
where
    T: DeserializeOwned,
{
    pub fn extend(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn next(&mut self) -> Result<Option<T>, MessageError> {
        let (message, read) = match take_from_bytes::<T>(&self.buffer).map_err(|error| error.into())
        {
            Ok((message, remaining)) => (message, self.buffer.len() - remaining.len()),
            Err(MessageError::UnexpectedEnd) => return Ok(None),
            Err(error) => return Err(error),
        };

        self.buffer = self.buffer.split_off(read);

        Ok(Some(message))
    }
}
