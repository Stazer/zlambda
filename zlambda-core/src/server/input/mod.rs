use crate::server::member::ServerMemberMessage;
use crate::server::{LogEntryData, LogEntryId, ServerId, ServerRegistrationMessageOutput};
use std::net::SocketAddr;
use tokio::net::TcpStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerSocketAcceptMessageInput {
    socket_stream: TcpStream,
    socket_address: SocketAddr,
}

impl From<ServerSocketAcceptMessageInput> for (TcpStream, SocketAddr) {
    fn from(input: ServerSocketAcceptMessageInput) -> Self {
        (input.socket_stream, input.socket_address)
    }
}

impl ServerSocketAcceptMessageInput {
    pub fn new(socket_stream: TcpStream, socket_address: SocketAddr) -> Self {
        Self {
            socket_stream,
            socket_address,
        }
    }

    pub fn socket_stream(&self) -> &TcpStream {
        &self.socket_stream
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRegistrationMessageInput {
    server_socket_address: SocketAddr,
}

impl From<ServerRegistrationMessageInput> for (SocketAddr,) {
    fn from(input: ServerRegistrationMessageInput) -> Self {
        (input.server_socket_address,)
    }
}

impl ServerRegistrationMessageInput {
    pub fn new(server_socket_address: SocketAddr) -> Self {
        Self {
            server_socket_address,
        }
    }

    pub fn server_socket_address(&self) -> SocketAddr {
        self.server_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerCommitRegistrationMessageInput {
    member_server_id: ServerId,
}

impl From<ServerCommitRegistrationMessageInput> for (ServerId,) {
    fn from(input: ServerCommitRegistrationMessageInput) -> Self {
        (input.member_server_id,)
    }
}

impl ServerCommitRegistrationMessageInput {
    pub fn new(member_server_id: ServerId) -> Self {
        Self { member_server_id }
    }

    pub fn member_server_id(&self) -> ServerId {
        self.member_server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRecoveryMessageInput {
    server_id: ServerId,
}

impl From<ServerRecoveryMessageInput> for (ServerId,) {
    fn from(input: ServerRecoveryMessageInput) -> Self {
        (input.server_id,)
    }
}

impl ServerRecoveryMessageInput {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogEntriesReplicationMessageInput {
    log_entries_data: Vec<LogEntryData>,
}

impl From<ServerLogEntriesReplicationMessageInput> for (Vec<LogEntryData>,) {
    fn from(input: ServerLogEntriesReplicationMessageInput) -> Self {
        (input.log_entries_data,)
    }
}

impl ServerLogEntriesReplicationMessageInput {
    pub fn new(log_entries_data: Vec<LogEntryData>) -> Self {
        Self { log_entries_data }
    }

    pub fn log_entries_data(&self) -> &Vec<LogEntryData> {
        &self.log_entries_data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogEntriesAcknowledgementMessageInput {
    log_entry_ids: Vec<LogEntryId>,
    server_id: ServerId,
}

impl From<ServerLogEntriesAcknowledgementMessageInput> for (Vec<LogEntryId>, ServerId) {
    fn from(input: ServerLogEntriesAcknowledgementMessageInput) -> Self {
        (input.log_entry_ids, input.server_id)
    }
}

impl ServerLogEntriesAcknowledgementMessageInput {
    pub fn new(log_entry_ids: Vec<LogEntryId>, server_id: ServerId) -> Self {
        Self {
            log_entry_ids,
            server_id,
        }
    }

    pub fn log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.log_entry_ids
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}
