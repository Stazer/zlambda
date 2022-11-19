use crate::log::{LogEntryData, LogEntryId};
use crate::node::{NodeId, Term};
use bytes::Bytes;
use postcard::{take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum MessageError {
    UnexpectedEnd,
    PostcardError(postcard::Error),
    IoError(io::Error),
}

impl Debug for MessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::PostcardError(error) => Debug::fmt(error, formatter),
            Self::IoError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for MessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::PostcardError(error) => Display::fmt(error, formatter),
            Self::IoError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for MessageError {}

impl From<postcard::Error> for MessageError {
    fn from(error: postcard::Error) -> Self {
        match error {
            postcard::Error::DeserializeUnexpectedEnd => Self::UnexpectedEnd,
            e => Self::PostcardError(e),
        }
    }
}

impl From<io::Error> for MessageError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClusterMessageRegisterResponse {
    Ok {
        id: NodeId,
        leader_id: NodeId,
        addresses: HashMap<NodeId, SocketAddr>,
        term: Term,
    },
    NotALeader {
        leader_address: SocketAddr,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClusterMessage {
    RegisterRequest {
        address: SocketAddr,
    },
    RegisterResponse(ClusterMessageRegisterResponse),
    AppendEntriesRequest {
        term: Term,
        log_entry_data: Vec<LogEntryData>,
    },
    AppendEntriesResponse {
        log_entry_ids: Vec<LogEntryId>,
    },
    RequestVoteRequest,
    RequestVoteResponse,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    RegisterRequest,
    RegisterResponse,
    DispatchRequest,
    DispatchResponse,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    Cluster(ClusterMessage),
    Client(ClientMessage),
}

impl Message {
    pub fn from_vec(bytes: &Vec<u8>) -> Result<(usize, Self), MessageError> {
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

#[derive(Clone, Debug, Default)]
pub struct MessageBufferReader {
    buffer: Vec<u8>,
}

impl MessageBufferReader {
    pub fn push(&mut self, bytes: Bytes) {
        self.buffer.extend(bytes);
    }

    pub fn next(&mut self) -> Result<Option<Message>, MessageError> {
        let (read, message) = match Message::from_vec(&self.buffer) {
            Ok((read, message)) => (read, message),
            Err(MessageError::UnexpectedEnd) => return Ok(None),
            Err(error) => return Err(error),
        };

        self.buffer.drain(0..read);

        Ok(Some(message))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageStreamReader {
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
        match self.buffer.next()? {
            Some(item) => Ok(Some(item)),
            None => {
                let bytes = match self.reader.next().await {
                    None => return Ok(None),
                    Some(Err(error)) => return Err(error.into()),
                    Some(Ok(bytes)) => bytes,
                };

                self.buffer.push(bytes);

                self.buffer.next()
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageStreamWriter {
    writer: OwnedWriteHalf,
}

impl MessageStreamWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self { writer }
    }

    pub async fn write(&mut self, message: &Message) -> Result<(), MessageError> {
        Ok(self.writer.write(&message.to_bytes()?).await.map(|_| ())?)
    }
}
