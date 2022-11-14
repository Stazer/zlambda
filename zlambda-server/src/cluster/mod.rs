pub mod node;

////////////////////////////////////////////////////////////////////////////////////////////////////

use bytes::Bytes;
use node::ClusterNodeId;
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

pub type TermId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ClusterMessageError {
    UnexpectedEnd,
    PostcardError(postcard::Error),
    IoError(io::Error),
}

impl Debug for ClusterMessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::PostcardError(error) => Debug::fmt(error, formatter),
            Self::IoError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for ClusterMessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::PostcardError(error) => Display::fmt(error, formatter),
            Self::IoError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for ClusterMessageError {}

impl From<postcard::Error> for ClusterMessageError {
    fn from(error: postcard::Error) -> Self {
        match error {
            postcard::Error::DeserializeUnexpectedEnd => Self::UnexpectedEnd,
            e => Self::PostcardError(e),
        }
    }
}

impl From<io::Error> for ClusterMessageError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub enum ClusterMessage {
    RegisterRequest { address: SocketAddr },
    RegisterResponse {
        node_id: ClusterNodeId,
        leader_node_id: ClusterNodeId,
        node_addresses: HashMap<ClusterNodeId, SocketAddr>,
        term_id: TermId,
    },
    AppendEntriesRequest,
    AppendEntriesResponse,
    RequestVoteRequest,
    RequestVoteResponse,
}

impl ClusterMessage {
    pub fn from_vec(bytes: &Vec<u8>) -> Result<(usize, Self), ClusterMessageError> {
        let (packet, remaining) = take_from_bytes::<Self>(bytes)?;
        Ok((bytes.len() - remaining.len(), packet))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, ClusterMessageError> {
        Ok(to_allocvec(&self)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes, ClusterMessageError> {
        Ok(Bytes::from(self.to_vec()?))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
pub struct ClusterMessageBufferReader {
    buffer: Vec<u8>,
}

impl ClusterMessageBufferReader {
    pub fn push(&mut self, bytes: Bytes) {
        self.buffer.extend(bytes);
    }

    pub fn next(&mut self) -> Result<Option<ClusterMessage>, ClusterMessageError> {
        let (read, message) = match ClusterMessage::from_vec(&self.buffer) {
            Ok((read, message)) => (read, message),
            Err(ClusterMessageError::UnexpectedEnd) => return Ok(None),
            Err(error) => return Err(error),
        };

        self.buffer.drain(0..read);

        Ok(Some(message))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterMessageStreamReader {
    buffer: ClusterMessageBufferReader,
    reader: ReaderStream<OwnedReadHalf>,
}

impl ClusterMessageStreamReader {
    pub fn new(reader: OwnedReadHalf) -> Self {
        Self {
            buffer: ClusterMessageBufferReader::default(),
            reader: ReaderStream::new(reader),
        }
    }

    pub async fn read(&mut self) -> Result<Option<ClusterMessage>, ClusterMessageError> {
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
pub struct ClusterMessageStreamWriter {
    writer: OwnedWriteHalf,
}

impl ClusterMessageStreamWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self { writer }
    }

    pub async fn write(&mut self, message: &ClusterMessage) -> Result<(), ClusterMessageError> {
        Ok(self.writer.write(&message.to_bytes()?).await.map(|_| ())?)
    }
}
