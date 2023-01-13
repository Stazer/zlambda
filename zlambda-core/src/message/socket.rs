use crate::message::{MessageBufferReader, MessageError};
use postcard::to_allocvec;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::marker::PhantomData;
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MessageSocketReader<T>
where
    T: DeserializeOwned,
{
    buffer: MessageBufferReader<T>,
    reader: ReaderStream<OwnedReadHalf>,
}

impl<T> MessageSocketReader<T>
where
    T: DeserializeOwned,
{
    pub fn new(reader: ReaderStream<OwnedReadHalf>) -> Self {
        Self {
            buffer: MessageBufferReader::<T>::default(),
            reader,
        }
    }

    pub async fn read(&mut self) -> Result<Option<T>, MessageError> {
        loop {
            match self.buffer.next()? {
                Some(item) => return Ok(Some(item)),
                None => {
                    let bytes = match self.reader.next().await {
                        None => return Ok(None),
                        Some(Err(error)) => return Err(error.into()),
                        Some(Ok(bytes)) => bytes,
                    };

                    self.buffer.extend(&bytes);
                }
            }
        }
    }

    pub async fn do_read(&mut self) -> Option<T> {
        match self.read().await {
            Ok(Some(message)) => Some(message),
            _ => None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MessageSocketWriter<T>
where
    T: Serialize,
{
    writer: BufWriter<OwnedWriteHalf>,
    r#type: PhantomData<T>,
}

impl<T> MessageSocketWriter<T>
where
    T: Serialize,
{
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer: BufWriter::new(writer),
            r#type: PhantomData::<T>,
        }
    }

    pub async fn write<I, M>(&mut self, message: I) -> Result<(), MessageError>
    where
        I: AsRef<M>,
        T: From<M>,
    {
        self.writer
            .write_all(&to_allocvec(T::from(message))?)
            .await
            .map(|_| ())?;

        Ok(())
    }
}
