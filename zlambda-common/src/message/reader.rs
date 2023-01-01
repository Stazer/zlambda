use crate::message::{Message, MessageError};
use bytes::BytesMut;
use std::fmt::Debug;
use std::marker::PhantomData;
use tokio::net::tcp::OwnedReadHalf;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

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

    pub fn into_inner(self) -> (BasicMessageBufferReader<T>, ReaderStream<OwnedReadHalf>) {
        (self.buffer, self.reader)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type BasicMessageStreamReaderTask<T> = BasicMessageStreamReaderMultiTask<T>;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::channel::DoSend;
use bytes::Bytes;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct BasicMessageStreamReaderSingleTask<T> {
    reader: BasicMessageStreamReader<T>,
}

impl<T> BasicMessageStreamReaderSingleTask<T>
where
    Result<T, MessageError>: From<Message>,
    T: Debug + Send + 'static,
{
    pub fn new(buffer: BasicMessageBufferReader<T>, reader: ReaderStream<OwnedReadHalf>) -> Self {
        Self {
            reader: BasicMessageStreamReader { buffer, reader },
        }
    }

    pub fn spawn(self) -> Receiver<Result<Option<T>, MessageError>> {
        let (sender, receiver) = channel(16);

        tokio::spawn(async move {
            self.run(sender).await;
        });

        receiver
    }

    async fn run(mut self, sender: Sender<Result<Option<T>, MessageError>>) {
        loop {
            let message = self.reader.read().await;

            let exit = matches!(message, Ok(None) | Err(_));

            sender.send(message).await.expect("");

            if exit {
                break;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BasicMessageStreamReaderMultiTask<T> {
    buffer: BasicMessageBufferReader<T>,
    reader: ReaderStream<OwnedReadHalf>,
}

impl<T> BasicMessageStreamReaderMultiTask<T>
where
    Result<T, MessageError>: From<Message>,
    T: Debug + Send + 'static,
{
    pub fn new(buffer: BasicMessageBufferReader<T>, reader: ReaderStream<OwnedReadHalf>) -> Self {
        Self { buffer, reader }
    }

    pub fn spawn(self) -> Receiver<Result<Option<T>, MessageError>> {
        let (message_sender, message_receiver) = channel::<Result<Option<T>, MessageError>>(16);
        let (bytes_sender, mut bytes_receiver) = channel::<Result<Bytes, MessageError>>(16);

        let (mut buffer, mut reader) = (self.buffer, self.reader);

        tokio::spawn(async move {
            'first: loop {
                let bytes = match bytes_receiver.recv().await {
                    None => {
                        message_sender.send(Ok(None)).await.expect("");
                        break;
                    }
                    Some(Err(error)) => {
                        message_sender.send(Err(error)).await.expect("");
                        break;
                    }
                    Some(Ok(bytes)) => bytes,
                };

                buffer.push(&bytes);

                'second: loop {
                    match buffer.next() {
                        Err(error) => {
                            message_sender.do_send(Err(error)).await;
                            break 'first;
                        }
                        Ok(Some(message)) => {
                            message_sender.do_send(Ok(Some(message))).await;
                        }
                        Ok(None) => break 'second,
                    }
                }
            }
        });

        tokio::spawn(async move {
            loop {
                match reader.next().await {
                    None => break,
                    Some(result) => {
                        let error = result.is_err();

                        bytes_sender
                            .do_send(result.map_err(|error| error.into()))
                            .await;

                        if error {
                            break;
                        }
                    }
                }
            }
        });

        message_receiver
    }
}
