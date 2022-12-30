use bytes::BytesMut;
use std::fmt::{Debug};
use std::marker::PhantomData;
use tokio::net::tcp::{OwnedReadHalf};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;
use crate::message::{Message, MessageError};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct BasicMessageBufferReader<T> {
    buffer: BytesMut,
    r#type: PhantomData<T>,
}

impl<T> Default for BasicMessageBufferReader<T> {
    fn default() -> Self {
        Self {
            buffer: BytesMut::default(),
            r#type: PhantomData::<T>,
        }
    }
}

impl<T> BasicMessageBufferReader<T>
where
    Result<T, MessageError>: From<Message>,
{
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn next(&mut self) -> Result<Option<T>, MessageError> {
        let (read, message) = match Message::from_bytes(&self.buffer) {
            Ok((read, message)) => (read, message),
            Err(MessageError::UnexpectedEnd) => return Ok(None),
            Err(error) => return Err(error),
        };

        self.buffer = self.buffer.split_off(read);

        Ok(Some(Result::<T, MessageError>::from(message)?))
    }

    pub fn into<S>(self) -> BasicMessageBufferReader<S>
    where
        Result<S, MessageError>: From<Message>,
    {
        BasicMessageBufferReader {
            buffer: self.buffer,
            r#type: PhantomData::<S>,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BasicMessageStreamReader<T> {
    buffer: BasicMessageBufferReader<T>,
    reader: ReaderStream<OwnedReadHalf>,
}

impl<T> BasicMessageStreamReader<T>
where
    Result<T, MessageError>: From<Message>,
{
    pub fn new(reader: OwnedReadHalf) -> Self {
        Self {
            buffer: BasicMessageBufferReader::<T>::default(),
            reader: ReaderStream::new(reader),
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

                    self.buffer.push(&bytes);
                }
            }
        }
    }

    pub fn into<S>(self) -> BasicMessageStreamReader<S>
    where
        Result<S, MessageError>: From<Message>,
    {
        BasicMessageStreamReader {
            buffer: self.buffer.into(),
            reader: self.reader,
        }
    }
}
