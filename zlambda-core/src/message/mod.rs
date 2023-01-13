////////////////////////////////////////////////////////////////////////////////////////////////////

/*use bytes::Bytes;
use postcard::{take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

mod error;
mod receiver;
mod sender;

pub use error::*;
pub use receiver::*;
pub use sender::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
}

impl Message {
    pub fn from_bytes(bytes: &[u8]) -> Result<(usize, Self), MessageError> {
        let (packet, remaining) = take_from_bytes::<Self>(bytes)?;
        Ok((bytes.len() - remaining.len(), packet))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, MessageError> {
        Ok(to_allocvec(&self)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes, MessageError> {
        Ok(Bytes::from(self.to_vec()?))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::message::{Message, MessageError};
use bytes::BytesMut;
use std::fmt::Debug;
use tokio::net::tcp::OwnedReadHalf;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;
use tokio::sync::{mpsc, oneshot};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
pub struct MessageBufferReader {
    buffer: BytesMut,
}

impl MessageBufferReader {
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn next(&mut self) -> Result<Option<Message>, MessageError> {
        let (read, message) = match Message::from_bytes(&self.buffer) {
            Ok((read, message)) => (read, message),
            Err(MessageError::UnexpectedEnd) => return Ok(None),
            Err(error) => return Err(error),
        };

        self.buffer = self.buffer.split_off(read);

        Ok(Some(message))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageSocketReceiver {
    buffer: MessageBufferReader,
    reader: ReaderStream<OwnedReadHalf>,
}

impl MessageStreamReader {
    pub fn new(reader: OwnedReadHalf) -> Self {
        Self {
            buffer: MessageBufferReader::default(),
            reader: ReaderStream::new(reader),
        }
    }

    pub async fn read(&mut self) -> Result<Option<Message>, MessageError> {
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
}

use crate::message::{Message, MessageError};
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio::net::tcp::OwnedWriteHalf;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageStreamWriter {
    writer: BufWriter<OwnedWriteHalf>,
}

impl MessageStreamWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer: BufWriter::new(writer),
        }
    }

    pub async fn write(&mut self, message: Message) -> Result<(), MessageError> {
        self.writer
            .write_all(&message.to_bytes()?)
            .await
            .map(|_| ())?;

        Ok(())
    }
}*/

mod buffer;
mod channel;
mod error;
mod socket;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use buffer::*;
pub use channel::*;
pub use error::*;
pub use socket::*;
