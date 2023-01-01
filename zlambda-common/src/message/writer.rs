use crate::message::{Message, MessageError};
use std::marker::PhantomData;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BasicMessageStreamWriter<T> {
    writer: OwnedWriteHalf,
    r#type: PhantomData<T>,
}

impl<T> BasicMessageStreamWriter<T>
where
    Message: From<T>,
{
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer,
            r#type: PhantomData::<T>,
        }
    }

    pub async fn write(&mut self, message: T) -> Result<(), MessageError> {
        Ok(self
            .writer
            .write_all(&Message::from(message).to_bytes()?)
            .await
            .map(|_| ())?)
    }

    pub fn into<S>(self) -> BasicMessageStreamWriter<S>
    where
        Message: From<S>,
    {
        BasicMessageStreamWriter {
            writer: self.writer,
            r#type: PhantomData::<S>,
        }
    }
}
