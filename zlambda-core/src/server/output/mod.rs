use crate::message::MessageQueueSender;
use crate::server::member::ServerMemberMessage;
use crate::server::{LogEntryId, LogTerm, ServerId};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRegistrationMessageNotALeaderOutput {
    leader_server_socket_address: SocketAddr,
}

impl From<ServerRegistrationMessageNotALeaderOutput> for (SocketAddr,) {
    fn from(output: ServerRegistrationMessageNotALeaderOutput) -> Self {
        (output.leader_server_socket_address,)
    }
}

impl ServerRegistrationMessageNotALeaderOutput {
    pub fn new(leader_server_socket_address: SocketAddr) -> Self {
        Self {
            leader_server_socket_address,
        }
    }

    pub fn leader_server_socket_address(&self) -> &SocketAddr {
        &self.leader_server_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRegistrationMessageSuccessOutput {
    server_id: ServerId,
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    current_log_term: LogTerm,
    member_sender: MessageQueueSender<ServerMemberMessage>,
}

impl From<ServerRegistrationMessageSuccessOutput>
    for (
        ServerId,
        ServerId,
        Vec<Option<SocketAddr>>,
        LogTerm,
        MessageQueueSender<ServerMemberMessage>,
    )
{
    fn from(output: ServerRegistrationMessageSuccessOutput) -> Self {
        (
            output.server_id,
            output.leader_server_id,
            output.server_socket_addresses,
            output.current_log_term,
            output.member_sender,
        )
    }
}

impl ServerRegistrationMessageSuccessOutput {
    pub fn new(
        server_id: ServerId,
        leader_server_id: ServerId,
        server_socket_addresses: Vec<Option<SocketAddr>>,
        current_log_term: LogTerm,
        member_sender: MessageQueueSender<ServerMemberMessage>,
    ) -> Self {
        Self {
            server_id,
            leader_server_id,
            server_socket_addresses,
            current_log_term,
            member_sender,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn leader_server_id(&self) -> ServerId {
        self.leader_server_id
    }

    pub fn server_socket_addresses(&self) -> &Vec<Option<SocketAddr>> {
        &self.server_socket_addresses
    }

    pub fn curremt_log_term(&self) -> LogTerm {
        self.current_log_term
    }

    pub fn member_sender(&self) -> &MessageQueueSender<ServerMemberMessage> {
        &self.member_sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerRegistrationMessageOutput {
    NotALeader(ServerRegistrationMessageNotALeaderOutput),
    Success(ServerRegistrationMessageSuccessOutput),
}

impl From<ServerRegistrationMessageNotALeaderOutput> for ServerRegistrationMessageOutput {
    fn from(output: ServerRegistrationMessageNotALeaderOutput) -> Self {
        Self::NotALeader(output)
    }
}

impl From<ServerRegistrationMessageSuccessOutput> for ServerRegistrationMessageOutput {
    fn from(output: ServerRegistrationMessageSuccessOutput) -> Self {
        Self::Success(output)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRecoveryMessageNotALeaderOutput {
    leader_server_socket_address: SocketAddr,
}

impl From<ServerRecoveryMessageNotALeaderOutput> for (SocketAddr,) {
    fn from(output: ServerRecoveryMessageNotALeaderOutput) -> Self {
        (output.leader_server_socket_address,)
    }
}

impl ServerRecoveryMessageNotALeaderOutput {
    pub fn new(leader_server_socket_address: SocketAddr) -> Self {
        Self {
            leader_server_socket_address,
        }
    }

    pub fn leader_server_socket_address(&self) -> &SocketAddr {
        &self.leader_server_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRecoveryMessageSuccessOutput {
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    current_log_term: LogTerm,
    member_queue_sender: MessageQueueSender<ServerMemberMessage>,
}

impl From<ServerRecoveryMessageSuccessOutput>
    for (
        ServerId,
        Vec<Option<SocketAddr>>,
        LogTerm,
        MessageQueueSender<ServerMemberMessage>,
    )
{
    fn from(output: ServerRecoveryMessageSuccessOutput) -> Self {
        (
            output.leader_server_id,
            output.server_socket_addresses,
            output.current_log_term,
            output.member_queue_sender,
        )
    }
}

impl ServerRecoveryMessageSuccessOutput {
    pub fn new(
        leader_server_id: ServerId,
        server_socket_addresses: Vec<Option<SocketAddr>>,
        current_log_term: LogTerm,
        member_queue_sender: MessageQueueSender<ServerMemberMessage>,
    ) -> Self {
        Self {
            leader_server_id,
            server_socket_addresses,
            current_log_term,
            member_queue_sender,
        }
    }

    pub fn leader_server_id(&self) -> ServerId {
        self.leader_server_id
    }

    pub fn server_socket_addresses(&self) -> &Vec<Option<SocketAddr>> {
        &self.server_socket_addresses
    }

    pub fn current_log_term(&self) -> LogTerm {
        self.current_log_term
    }

    pub fn member_queue_sender(&self) -> &MessageQueueSender<ServerMemberMessage> {
        &self.member_queue_sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerRecoveryMessageOutput {
    NotALeader(ServerRecoveryMessageNotALeaderOutput),
    IsOnline,
    Unknown,
    Success(ServerRecoveryMessageSuccessOutput),
}

impl From<ServerRecoveryMessageNotALeaderOutput> for ServerRecoveryMessageOutput {
    fn from(output: ServerRecoveryMessageNotALeaderOutput) -> Self {
        Self::NotALeader(output)
    }
}

impl From<ServerRecoveryMessageSuccessOutput> for ServerRecoveryMessageOutput {
    fn from(output: ServerRecoveryMessageSuccessOutput) -> Self {
        Self::Success(output)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogEntriesReplicationMessageOutput {
    log_entry_ids: Vec<LogEntryId>,
}

impl From<ServerLogEntriesReplicationMessageOutput> for (Vec<LogEntryId>,) {
    fn from(output: ServerLogEntriesReplicationMessageOutput) -> Self {
        (output.log_entry_ids,)
    }
}

impl ServerLogEntriesReplicationMessageOutput {
    pub fn new(log_entry_ids: Vec<LogEntryId>) -> Self {
        Self { log_entry_ids }
    }

    pub fn log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.log_entry_ids
    }
}
