use crate::dispatch::DispatchId;
use crate::log::{LogEntryData, LogEntryId};
use crate::module::ModuleId;
use crate::node::NodeId;
use crate::term::Term;
use bytes::Bytes;
use bytes::BytesMut;
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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderToGuestMessage {
    RegisterOkResponse {
        id: NodeId,
        leader_id: NodeId,
        addresses: HashMap<NodeId, SocketAddr>,
        term: Term,
    },
    HandshakeErrorResponse {
        message: String,
    },
    HandshakeOkResponse {
        leader_id: NodeId,
    },
}

impl From<Message> for Result<LeaderToGuestMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::LeaderToGuest(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

pub type LeaderToGuestMessageStreamReader = BasicMessageStreamReader<LeaderToGuestMessage>;
pub type LeaderToGuestMessageStreamWriter = BasicMessageStreamWriter<LeaderToGuestMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GuestToLeaderMessage {}

impl From<Message> for Result<GuestToLeaderMessage, MessageError> {
    fn from(message: Message) -> Self {
        Err(MessageError::UnexpectedMessage(message))
    }
}

pub type GuestToLeaderMessageStreamReader = BasicMessageStreamReader<GuestToLeaderMessage>;
pub type GuestToLeaderMessageStreamWriter = BasicMessageStreamWriter<GuestToLeaderMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GuestToNodeMessage {
    RegisterRequest {
        address: SocketAddr,
    },
    HandshakeRequest {
        address: SocketAddr,
        node_id: NodeId,
    },
}

impl From<Message> for Result<GuestToNodeMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::GuestToNode(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

pub type GuestToNodeMessageStreamReader = BasicMessageStreamReader<GuestToNodeMessage>;
pub type GuestToNodeMessageStreamWriter = BasicMessageStreamWriter<GuestToNodeMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToGuestMessage {
    RegisterNotALeaderResponse { leader_address: SocketAddr },
    HandshakeNotALeaderResponse { leader_address: SocketAddr },
}

impl From<Message> for Result<FollowerToGuestMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::FollowerToGuest(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToLeaderMessage {
    AppendEntriesResponse {
        log_entry_ids: Vec<LogEntryId>,
    },
}

impl From<Message> for Result<FollowerToLeaderMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::FollowerToLeader(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

pub type FollowerToLeaderMessageStreamReader = BasicMessageStreamReader<FollowerToLeaderMessage>;
pub type FollowerToLeaderMessageStreamWriter = BasicMessageStreamWriter<FollowerToLeaderMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderToFollowerMessage {
    AppendEntriesRequest {
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    },
}

impl From<Message> for Result<LeaderToFollowerMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::LeaderToFollower(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

pub type LeaderToFollowerMessageStreamReader = BasicMessageStreamReader<LeaderToFollowerMessage>;
pub type LeaderToFollowerMessageStreamWriter = BasicMessageStreamWriter<LeaderToFollowerMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientToNodeMessage {
    InitializeRequest,
    Append {
        module_id: ModuleId,
        bytes: Bytes,
    },
    LoadRequest {
        module_id: ModuleId,
    },
    /*ApplyRequest {
        module_id: ModuleId,
    },*/
    DispatchRequest {
        //node_id: Option<NodeId>,
        module_id: ModuleId,
        dispatch_id: DispatchId,
        payload: Vec<u8>,
    },
}

impl From<Message> for Result<ClientToNodeMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::ClientToNode(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

pub type ClientToNodeMessageStreamReader = BasicMessageStreamReader<ClientToNodeMessage>;
pub type ClientToNodeMessageStreamWriter = BasicMessageStreamWriter<ClientToNodeMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeToClientMessage {
    InitializeResponse {
        module_id: ModuleId,
    },
    LoadResponse {
        module_id: ModuleId,
        result: Result<(), String>,
    },
    ApplyResponse {
        module_id: ModuleId,
        result: Result<(), String>,
    },
    DispatchResponse {
        dispatch_id: DispatchId,
        result: Result<Vec<u8>, String>,
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

pub type NodeToClientMessageStreamReader = BasicMessageStreamReader<NodeToClientMessage>;
pub type NodeToClientMessageStreamWriter = BasicMessageStreamWriter<NodeToClientMessage>;

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

pub type CandidateToCandidateMessageStreamReader =
    BasicMessageStreamReader<CandidateToCandidateMessage>;
pub type CandidateToCandidateMessageStreamWriter =
    BasicMessageStreamWriter<CandidateToCandidateMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    GuestToLeader(GuestToLeaderMessage),
    LeaderToGuest(LeaderToGuestMessage),

    FollowerToGuest(FollowerToGuestMessage),

    GuestToNode(GuestToNodeMessage),

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

impl From<GuestToLeaderMessage> for Message {
    fn from(message: GuestToLeaderMessage) -> Self {
        Self::GuestToLeader(message)
    }
}

impl From<LeaderToGuestMessage> for Message {
    fn from(message: LeaderToGuestMessage) -> Self {
        Self::LeaderToGuest(message)
    }
}

impl From<FollowerToGuestMessage> for Message {
    fn from(message: FollowerToGuestMessage) -> Self {
        Self::FollowerToGuest(message)
    }
}

impl From<GuestToNodeMessage> for Message {
    fn from(message: GuestToNodeMessage) -> Self {
        Self::GuestToNode(message)
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

pub type MessageStreamReader = BasicMessageStreamReader<Message>;
pub type MessageStreamWriter = BasicMessageStreamWriter<Message>;

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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BasicMessageStreamWriter<T> {
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

    pub fn into<S>(self) -> BasicMessageStreamWriter<S>
    where
        Message: From<S>,
    {
        BasicMessageStreamWriter {
            writer: self.writer,
            r#type: PhantomData::<S>,
        }
    }
}
