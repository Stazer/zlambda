use crate::common::bytes::Bytes;
use crate::common::message::{AsynchronousMessage, MessageBufferReader, MessageError};
use crate::common::net::tcp::OwnedReadHalf;
use crate::common::net::tcp::OwnedWriteHalf;
use postcard::to_allocvec;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::marker::PhantomData;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageSocketSender<T>
where
    T: Serialize,
{
    writer: OwnedWriteHalf,
    r#type: PhantomData<T>,
}

impl<T> MessageSocketSender<T>
where
    T: Serialize,
{
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer,
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

    pub async fn send_raw(&mut self, bytes: Bytes) -> Result<(), MessageError> {
        self.writer.write_all(&bytes).await.map(|_| ())?;

        Ok(())
    }

    pub async fn send_asynchronous<I>(&mut self, input: I) -> Result<(), MessageError>
    where
        T: From<AsynchronousMessage<I>>,
    {
        self.send(AsynchronousMessage::new(input)).await
    }

    /*pub async fn do_send_synchronous<I, O>(&self, input: I) -> O
    where
        T: From<SynchronousMessage<I, O>>,
        O: Debug + Send,
    {
        self.do_send(SynchronousMessage::new(input, sender)).await;

        receiver.do_receive().await
    }*/
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
