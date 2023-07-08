mod id;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use id::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::future::{Stream, StreamExt};
use crate::common::message::MessageQueueReceiver;
use crate::common::message::{message_queue, MessageQueueSender};
use crate::common::utility::{Bytes, BytesMut};
use postcard::{take_from_bytes, to_allocvec};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::{self, Display, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NotificationBodyItem = Bytes;

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
    JsonError(serde_json::Error),
    UnexpectedEnd,
}

impl Display for NotificationError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::PostcardError(error) => Display::fmt(error, formatter),
            Self::JsonError(error) => Display::fmt(error, formatter),
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
        }
    }
}

impl Error for NotificationError {}

impl From<postcard::Error> for NotificationError {
    fn from(error: postcard::Error) -> Self {
        match error {
            postcard::Error::DeserializeUnexpectedEnd => Self::UnexpectedEnd,
            e => Self::PostcardError(e),
        }
    }
}

impl From<serde_json::Error> for NotificationError {
    fn from(error: serde_json::Error) -> Self {
        Self::JsonError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait NotificationBodyItemStreamExt: Stream<Item = NotificationBodyItem> {
    fn deserializer(self) -> NotificationBodyStreamDeserializer<Self>
    where
        Self: Sized + Unpin,
    {
        NotificationBodyStreamDeserializer::new(self)
    }

    fn serializer(self) -> NotificationBodyStreamSerializer<Self>
    where
        Self: Sized + Unpin,
    {
        NotificationBodyStreamSerializer::new(self)
    }
}

impl<T> NotificationBodyItemStreamExt for T where T: Stream<Item = NotificationBodyItem> {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NotificationBodyStreamDeserializer<T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    stream: T,
    buffer: BytesMut,
}

impl<T> Stream for NotificationBodyStreamDeserializer<T>
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

impl<T> NotificationBodyStreamDeserializer<T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            buffer: BytesMut::default(),
        }
    }

    pub async fn deserialize<S>(&mut self) -> Result<S, NotificationError>
    where
        S: DeserializeOwned + std::fmt::Debug,
    {
        loop {
            let bytes = match self.stream.next().await {
                None => return Err(NotificationError::UnexpectedEnd),
                Some(bytes) => bytes,
            };

            self.buffer.extend_from_slice(&bytes);

            let (bytes, read) = match take_from_bytes::<NotificationBodyItemType>(&self.buffer)
                .map_err(NotificationError::from)
            {
                Ok((bytes, remaining)) => (bytes, self.buffer.len() - remaining.len()),
                Err(NotificationError::UnexpectedEnd) => continue,
                Err(error) => return Err(error),
            };

            self.buffer = self.buffer.split_off(read);

            // TODO

            let notification = match bytes {
                NotificationBodyItemType::Binary(bytes) => take_from_bytes::<S>(&bytes)?.0,
                NotificationBodyItemType::Json(bytes) => from_slice::<S>(&bytes)?,
            };

            return Ok(notification);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NotificationBodyStreamSerializer<T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    stream: T,
    buffer: BytesMut,
}

impl<T> Stream for NotificationBodyStreamSerializer<T>
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

impl<T> NotificationBodyStreamSerializer<T>
where
    T: Stream<Item = NotificationBodyItem> + Unpin,
{
    pub(crate) fn new(stream: T) -> Self {
        Self {
            stream,
            buffer: BytesMut::default(),
        }
    }

    pub fn serialize<S>(&mut self, value: &S) -> Result<(), NotificationError>
    where
        S: Serialize,
    {
        self.serialize_binary(Bytes::from(to_allocvec(value)?))?;

        Ok(())
    }

    pub fn serialize_binary(&mut self, value: Bytes) -> Result<(), NotificationError> {
        self.buffer
            .extend_from_slice(&to_allocvec(&NotificationBodyItemType::Binary(
                Bytes::from(to_allocvec(&value)?),
            ))?);

        Ok(())
    }

    pub fn serialize_json(&mut self, value: Bytes) -> Result<(), NotificationError> {
        self.buffer
            .extend_from_slice(&to_allocvec(&NotificationBodyItemType::Json(value))?);

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub enum NotificationBodyItemType {
    Json(Bytes),
    Binary(Bytes),
}
