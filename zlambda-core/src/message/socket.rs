use crate::message::{AsynchronousMessage, MessageBufferReader, MessageError};
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

#[derive(Debug)]
pub struct MessageSocketSender<T>
where
    T: Serialize,
{
    writer: BufWriter<OwnedWriteHalf>,
    r#type: PhantomData<T>,
}

impl<T> MessageSocketSender<T>
where
    T: Serialize,
{
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer: BufWriter::new(writer),
            r#type: PhantomData::<T>,
        }
    }

    pub async fn send<M>(&mut self, message: M) -> Result<(), MessageError>
    where
        T: From<M>,
    {
        self.writer
            .write_all(&to_allocvec(&T::from(message))?)
            .await
            .map(|_| ())?;

        Ok(())
    }

    pub async fn send_asynchronous<I>(&mut self, input: I) -> Result<(), MessageError>
    where
        T: From<AsynchronousMessage<I>>,
    {
        self.send(AsynchronousMessage::new(input)).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageSocketReceiver<T>
where
    T: DeserializeOwned,
{
    buffer: MessageBufferReader<T>,
    reader: ReaderStream<OwnedReadHalf>,
}

impl<T> MessageSocketReceiver<T>
where
    T: DeserializeOwned,
{
    pub fn new(reader: OwnedReadHalf) -> Self {
        Self {
            buffer: MessageBufferReader::<T>::default(),
            reader: ReaderStream::new(reader),
        }
    }

    pub async fn receive(&mut self) -> Result<Option<T>, MessageError> {
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
}
