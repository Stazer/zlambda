use crate::message::{Message, MessageError};
use std::marker::PhantomData;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::io::BufWriter;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BasicMessageStreamWriter<T> {
    writer: BufWriter<OwnedWriteHalf>,
    r#type: PhantomData<T>,
}

impl<T> BasicMessageStreamWriter<T>
where
    Message: From<T>,
{
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer: BufWriter::with_capacity(1024, writer),
            r#type: PhantomData::<T>,
        }
    }

    pub async fn write(&mut self, message: T) -> Result<(), MessageError> {
        self
            .writer
            .write_all(&Message::from(message).to_bytes()?)
            .await
            .map(|_| ())?;

        self.writer.flush().await?;

        Ok(())
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
