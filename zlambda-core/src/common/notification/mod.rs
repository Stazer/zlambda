use crate::common::future::{Sink, SinkExt, Stream, StreamExt};
use crate::common::message::MessageQueueReceiver;
use crate::common::message::{message_queue, MessageQueueSender};
use crate::common::utility::{Bytes, BytesMut};
use postcard::{take_from_bytes, to_allocvec};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::pin::Pin;
use std::task::{Context, Poll};

////////////////////////////////////////////////////////////////////////////////////////////////////

/*pub struct NotificationHeader<T> {
    data: T,
}

impl<T> NotificationHeader<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn write(&self) -> Result<Bytes, NotificationError>
    where
        T: Serialize,
    {
        Ok(Bytes::from(to_allocvec(&self.data)?))
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NotificationBodyItem = Bytes;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NotificationId = usize;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NotificationBodyItemQueueSender = MessageQueueSender<NotificationBodyItem>;
pub type NotificationBodyItemQueueReceiver = MessageQueueReceiver<NotificationBodyItem>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn notification_body_item_queue() -> (
    NotificationBodyItemQueueSender,
    NotificationBodyItemQueueReceiver,
) {
    message_queue::<NotificationBodyItem>()
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NotificationError {
    PostcardError(postcard::Error),
    UnexpectedEnd,
}

impl From<postcard::Error> for NotificationError {
    fn from(error: postcard::Error) -> Self {
        match error {
            postcard::Error::DeserializeUnexpectedEnd => Self::UnexpectedEnd,
            e => Self::PostcardError(e),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait NotificationBodyItemStreamExt: Stream<Item = NotificationBodyItem> {
    fn reader(&mut self) -> NotificationBodyStreamReader<'_, Self>
    where
        Self: Sized + Unpin,
    {
        NotificationBodyStreamReader::new(self)
    }

    fn writer(&mut self) -> NotificationBodyStreamWriter<'_, Self>
    where
        Self: Sized + Unpin,
    {
        NotificationBodyStreamWriter::new(self)
    }
}

impl<T> NotificationBodyItemStreamExt for T where T: Stream<Item = NotificationBodyItem> {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NotificationBodyStreamReader<'a, T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    stream: &'a mut T,
    buffer: BytesMut,
}

impl<'a, T> Stream for NotificationBodyStreamReader<'a, T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    type Item = NotificationBodyItem;

    fn poll_next(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<NotificationBodyItem>> {
        if !self.buffer.is_empty() {
            return Poll::Ready(Some(self.buffer.split().freeze()));
        }

        match Pin::new(&mut self.stream).poll_next(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(item) => Poll::Ready(item),
        }
    }
}

impl<'a, T> NotificationBodyStreamReader<'a, T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    pub fn new(stream: &'a mut T) -> Self {
        Self {
            stream,
            buffer: BytesMut::default(),
        }
    }

    pub async fn deserialize<S>(&mut self) -> Result<S, NotificationError>
    where
        S: DeserializeOwned,
    {
        loop {
            let bytes = match self.stream.next().await {
                None => return Err(NotificationError::UnexpectedEnd),
                Some(bytes) => bytes,
            };

            self.buffer.extend_from_slice(&bytes);

            let (notification, read) =
                match take_from_bytes::<S>(&self.buffer).map_err(NotificationError::from) {
                    Ok((notification, remaining)) => {
                        (notification, self.buffer.len() - remaining.len())
                    }
                    Err(NotificationError::UnexpectedEnd) => continue,
                    Err(error) => return Err(error),
                };

            self.buffer = self.buffer.split_off(read);

            return Ok(notification);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NotificationBodyStreamWriter<'a, T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    stream: &'a mut T,
    buffer: BytesMut,
}

impl<'a, T> NotificationBodyStreamWriter<'a, T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    pub(crate) fn new(
        stream: &'a mut T,
    ) -> Self {
        Self {
            stream,
            buffer: BytesMut::default(),
        }
    }

    pub fn serialize<S>(&mut self, value: &S) -> Result<(), NotificationError>
    where
        S: Serialize,
    {
        self.buffer.extend_from_slice(&to_allocvec(&value)?);

        Ok(())
    }
}

impl<'a, T> Stream for NotificationBodyStreamWriter<'a, T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    type Item = NotificationBodyItem;

    fn poll_next(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<NotificationBodyItem>> {
        if !self.buffer.is_empty() {
            return Poll::Ready(Some(self.buffer.split().freeze()));
        }

        match Pin::new(&mut self.stream).poll_next(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(item) => Poll::Ready(item),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait NotificationBodyItemSinkExt: Sink<NotificationBodyItem> {
    fn writer(&mut self) -> NotificationBodySinkWriter<'_, Self>
    where
        Self: Sized + Unpin,
    {
        NotificationBodySinkWriter::new(self)
    }
}

impl<T> NotificationBodyItemSinkExt for T where T: Sink<NotificationBodyItem> {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NotificationBodySinkWriter<'a, T>
where
    T: Sink<NotificationBodyItem> + Unpin,
{
    sink: &'a mut T,
}

impl<'a, T> NotificationBodySinkWriter<'a, T>
where
    T: Sink<NotificationBodyItem> + Unpin,
{
    pub(crate) fn new(sink: &'a mut T) -> Self {
        Self { sink }
    }

    pub async fn serialize<S>(&mut self, value: &S) -> Result<(), NotificationError>
    where
        S: Serialize,
    {
        self.sink.send(Bytes::from(to_allocvec(&value)?)).await;

        Ok(())
    }
}
