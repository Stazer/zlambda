use crate::dispatch::DispatchId;
use crate::log::{LogEntryData, LogEntryId};
use crate::module::ModuleId;
use crate::node::NodeId;
use crate::term::Term;
use bytes::Bytes;
use postcard::{take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::marker::PhantomData;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum MessageError {
    UnexpectedEnd,
    UnexpectedMessage(Message),
    PostcardError(postcard::Error),
    IoError(io::Error),
}

impl Debug for MessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::UnexpectedMessage(_) => write!(formatter, "Unexpected message"),
            Self::PostcardError(error) => Debug::fmt(error, formatter),
            Self::IoError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for MessageError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedEnd => write!(formatter, "Unexpected end"),
            Self::UnexpectedMessage(_) => write!(formatter, "Unexpected message"),
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

impl From<std::convert::Infallible> for MessageError {
    fn from(v: std::convert::Infallible) -> Self {
        panic!()
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
        last_committed_log_entry_id: Option<LogEntryId>,
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

    InitializeRequest,
    InitializeResponse(ModuleId),
    Append(ModuleId, Vec<u8>),
    LoadRequest(ModuleId),
    LoadResponse(Result<ModuleId, String>),

    DispatchRequest(ModuleId, DispatchId, Vec<u8>),
    DispatchResponse(DispatchId, Result<Vec<u8>, String>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderToFollowerMessage {
    RegisterResponse {},
}

impl From<Message> for Result<LeaderToFollowerMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::LeaderToFollower(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToLeaderMessage {
    RegisterRequest,
}

impl From<Message> for Result<FollowerToLeaderMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::FollowerToLeader(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientToNodeMessage {
    RegisterRequest,
    InitializeRequest,
    Append { module_id: ModuleId, bytes: Vec<u8> },
    LoadRequest { module_id: ModuleId },
}

impl From<Message> for Result<ClientToNodeMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::ClientToNode(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeToClientMessage {
    RegisterResponse,
    InitializeResponse {
        module_id: ModuleId,
    },
    LoadResponse {
        module_id: ModuleId,
        result: Result<(), String>,
    },
}

impl From<Message> for Result<NodeToClientMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::NodeToClient(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CandidateToCandidateMessage {}

impl From<Message> for Result<CandidateToCandidateMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::CandidateToCandidate(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    Cluster(ClusterMessage),
    Client(ClientMessage),

    LeaderToFollower(LeaderToFollowerMessage),
    FollowerToLeader(FollowerToLeaderMessage),

    ClientToNode(ClientToNodeMessage),
    NodeToClient(NodeToClientMessage),

    CandidateToCandidate(CandidateToCandidateMessage),
}

impl From<Message> for Result<Message, MessageError> {
    fn from(message: Message) -> Self {
        Ok(message)
    }
}

impl From<LeaderToFollowerMessage> for Message {
    fn from(message: LeaderToFollowerMessage) -> Self {
        Self::LeaderToFollower(message)
    }
}

impl From<FollowerToLeaderMessage> for Message {
    fn from(message: FollowerToLeaderMessage) -> Self {
        Self::FollowerToLeader(message)
    }
}

impl From<NodeToClientMessage> for Message {
    fn from(message: NodeToClientMessage) -> Self {
        Self::NodeToClient(message)
    }
}

impl From<ClientToNodeMessage> for Message {
    fn from(message: ClientToNodeMessage) -> Self {
        Self::ClientToNode(message)
    }
}

impl From<CandidateToCandidateMessage> for Message {
    fn from(message: CandidateToCandidateMessage) -> Self {
        Self::CandidateToCandidate(message)
    }
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

#[derive(Clone, Debug)]
pub struct BasicMessageBufferReader<T> {
    buffer: Vec<u8>,
    r#type: PhantomData<T>,
}

impl<T> Default for BasicMessageBufferReader<T> {
    fn default() -> Self {
        Self {
            buffer: Vec::default(),
            r#type: PhantomData::<T>,
        }
    }
}

impl<T> BasicMessageBufferReader<T>
where
    Result<T, MessageError>: From<Message>,
{
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes);
    }

    pub fn next(&mut self) -> Result<Option<T>, MessageError> {
        let (read, message) = match Message::from_vec(&self.buffer) {
            Ok((read, message)) => (read, message),
            Err(error) => return Err(error),
        };

        self.buffer.drain(0..read);

        Ok(Some(Result::<T, MessageError>::from(message)?))
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BasicMessageStreamWriter<T>
where
    Message: From<T>,
{
    writer: OwnedWriteHalf,
    r#type: PhantomData<T>,
}

impl<T> BasicMessageStreamWriter<T>
where
    Message: From<T>,
{
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer,
            r#type: PhantomData::<T>,
        }
    }

    pub async fn write(&mut self, message: T) -> Result<(), MessageError> {
        Ok(self
            .writer
            .write_all(&Message::from(message).to_bytes()?)
            .await
            .map(|_| ())?)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LeaderToFollowerMessageStreamReader = BasicMessageStreamReader<LeaderToFollowerMessage>;
pub type LeaderToFollowerMessageStreamWriter = BasicMessageStreamWriter<LeaderToFollowerMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type FollowerToLeaderMessageStreamReader = BasicMessageStreamReader<FollowerToLeaderMessage>;
pub type FollowerToLeaderMessageStreamWriter = BasicMessageStreamWriter<FollowerToLeaderMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientToNodeMessageStreamReader = BasicMessageStreamReader<ClientToNodeMessage>;
pub type ClientToNodeMessageStreamWriter = BasicMessageStreamWriter<ClientToNodeMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeToClientMessageStreamReader = BasicMessageStreamReader<NodeToClientMessage>;
pub type NodeToClientMessageStreamWriter = BasicMessageStreamWriter<NodeToClientMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type CandidateToCandidateMessageStreamReader =
    BasicMessageStreamReader<CandidateToCandidateMessage>;
pub type CandidateToCandidateMessageStreamWriter =
    BasicMessageStreamWriter<CandidateToCandidateMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type MessageBufferReader = BasicMessageBufferReader<Message>;
pub type MessageStreamReader = BasicMessageStreamReader<Message>;
pub type MessageStreamWriter = BasicMessageStreamWriter<Message>;
