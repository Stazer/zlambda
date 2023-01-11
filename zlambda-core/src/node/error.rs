use crate::error::RecoveryError;
use crate::message::{MessageError, MessageStreamReader, MessageStreamWriter};
use std::io;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum CreateNodeError {
    IoError(io::Error),
    Message(MessageError),
    Recovery(RecoveryError),
}

impl From<io::Error> for CreateNodeError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<MessageError> for CreateNodeError {
    fn from(error: MessageError) -> Self {
        Self::Message(error)
    }
}

impl From<RecoveryError> for CreateNodeError {
    fn from(error: RecoveryError) -> Self {
        Self::Recovery(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollowerRegistrationNotALeaderError {
    leader_address: SocketAddr,
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
}

impl From<NodeFollowerRegistrationNotALeaderError> for (SocketAddr, MessageStreamReader, MessageStreamWriter) {
    fn from(error: NodeFollowerRegistrationNotALeaderError) -> Self {
        (error.leader_address, error.reader, error.writer)
    }
}

impl From<NodeFollowerRegistrationNotALeaderError> for NodeFollowerRegistrationError {
    fn from(error: NodeFollowerRegistrationNotALeaderError) -> Self {
        Self::NotALeader(error)
    }
}

impl NodeFollowerRegistrationNotALeaderError {
    pub fn new(leader_address: SocketAddr,
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    ) -> Self {
        Self { leader_address, reader, writer }
    }

    pub fn leader_address(&self) -> &SocketAddr {
        &self.leader_address
    }

    pub fn reader(&self) -> &MessageStreamReader {
        &self.reader
    }

    pub fn writer(&self) -> &MessageStreamWriter {
        &self.writer
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeFollowerRegistrationError {
    NotALeader(NodeFollowerRegistrationNotALeaderError),
}
