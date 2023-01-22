use crate::server::{LogEntry, LogEntryId, LogTerm, ServerId};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRegistrationRequestMessageInput {
    server_socket_address: SocketAddr,
}

impl From<GeneralRegistrationRequestMessageInput> for (SocketAddr,) {
    fn from(input: GeneralRegistrationRequestMessageInput) -> Self {
        (input.server_socket_address,)
    }
}

impl GeneralRegistrationRequestMessageInput {
    pub fn new(server_socket_address: SocketAddr) -> Self {
        Self {
            server_socket_address,
        }
    }

    pub fn server_socket_address(&self) -> &SocketAddr {
        &self.server_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRegistrationResponseMessageNotALeaderInput {
    leader_server_socket_address: SocketAddr,
}

impl From<GeneralRegistrationResponseMessageNotALeaderInput> for (SocketAddr,) {
    fn from(input: GeneralRegistrationResponseMessageNotALeaderInput) -> Self {
        (input.leader_server_socket_address,)
    }
}

impl GeneralRegistrationResponseMessageNotALeaderInput {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRegistrationResponseMessageSuccessInput {
    server_id: ServerId,
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    log_term: LogTerm,
}

impl From<GeneralRegistrationResponseMessageSuccessInput>
    for (ServerId, ServerId, Vec<Option<SocketAddr>>, LogTerm)
{
    fn from(input: GeneralRegistrationResponseMessageSuccessInput) -> Self {
        (
            input.server_id,
            input.leader_server_id,
            input.server_socket_addresses,
            input.log_term,
        )
    }
}

impl GeneralRegistrationResponseMessageSuccessInput {
    pub fn new(
        server_id: ServerId,
        leader_server_id: ServerId,
        server_socket_addresses: Vec<Option<SocketAddr>>,
        log_term: LogTerm,
    ) -> Self {
        Self {
            server_id,
            leader_server_id,
            server_socket_addresses,
            log_term,
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

    pub fn log_term(&self) -> LogTerm {
        self.log_term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralRegistrationResponseMessageInput {
    NotALeader(GeneralRegistrationResponseMessageNotALeaderInput),
    Success(GeneralRegistrationResponseMessageSuccessInput),
}

impl From<GeneralRegistrationResponseMessageNotALeaderInput>
    for GeneralRegistrationResponseMessageInput
{
    fn from(input: GeneralRegistrationResponseMessageNotALeaderInput) -> Self {
        Self::NotALeader(input)
    }
}

impl From<GeneralRegistrationResponseMessageSuccessInput>
    for GeneralRegistrationResponseMessageInput
{
    fn from(input: GeneralRegistrationResponseMessageSuccessInput) -> Self {
        Self::Success(input)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRecoveryRequestMessageInput {
    server_id: ServerId,
}

impl From<GeneralRecoveryRequestMessageInput> for (ServerId,) {
    fn from(input: GeneralRecoveryRequestMessageInput) -> Self {
        (input.server_id,)
    }
}

impl GeneralRecoveryRequestMessageInput {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> &ServerId {
        &self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRecoveryResponseMessageNotALeaderInput {
    leader_socket_address: SocketAddr,
}

impl From<GeneralRecoveryResponseMessageNotALeaderInput> for (SocketAddr,) {
    fn from(input: GeneralRecoveryResponseMessageNotALeaderInput) -> Self {
        (input.leader_socket_address,)
    }
}

impl GeneralRecoveryResponseMessageNotALeaderInput {
    pub fn new(leader_socket_address: SocketAddr) -> Self {
        Self {
            leader_socket_address,
        }
    }

    pub fn leader_socket_address(&self) -> &SocketAddr {
        &self.leader_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRecoveryResponseMessageSuccessInput {
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    log_term: LogTerm,
}

impl From<GeneralRecoveryResponseMessageSuccessInput>
    for (ServerId, Vec<Option<SocketAddr>>, LogTerm)
{
    fn from(input: GeneralRecoveryResponseMessageSuccessInput) -> Self {
        (
            input.leader_server_id,
            input.server_socket_addresses,
            input.log_term,
        )
    }
}

impl GeneralRecoveryResponseMessageSuccessInput {
    pub fn new(
        leader_server_id: ServerId,
        server_socket_addresses: Vec<Option<SocketAddr>>,
        log_term: LogTerm,
    ) -> Self {
        Self {
            leader_server_id,
            server_socket_addresses,
            log_term,
        }
    }

    pub fn leader_server_id(&self) -> ServerId {
        self.leader_server_id
    }

    pub fn server_socket_addresses(&self) -> &Vec<Option<SocketAddr>> {
        &self.server_socket_addresses
    }

    pub fn log_term(&self) -> LogTerm {
        self.log_term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralRecoveryResponseMessageInput {
    NotALeader(GeneralRecoveryResponseMessageNotALeaderInput),
    IsOnline,
    Unknown,
    Success(GeneralRecoveryResponseMessageSuccessInput),
}

impl From<GeneralRecoveryResponseMessageNotALeaderInput> for GeneralRecoveryResponseMessageInput {
    fn from(input: GeneralRecoveryResponseMessageNotALeaderInput) -> Self {
        Self::NotALeader(input)
    }
}

impl From<GeneralRecoveryResponseMessageSuccessInput> for GeneralRecoveryResponseMessageInput {
    fn from(input: GeneralRecoveryResponseMessageSuccessInput) -> Self {
        Self::Success(input)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralLogEntriesAppendRequestMessageInput {
    log_entries: Vec<LogEntry>,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_current_term: LogTerm,
}

impl From<GeneralLogEntriesAppendRequestMessageInput>
    for (Vec<LogEntry>, Option<LogEntryId>, LogTerm)
{
    fn from(input: GeneralLogEntriesAppendRequestMessageInput) -> Self {
        (
            input.log_entries,
            input.last_committed_log_entry_id,
            input.log_current_term,
        )
    }
}

impl GeneralLogEntriesAppendRequestMessageInput {
    pub fn new(
        log_entries: Vec<LogEntry>,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_current_term: LogTerm,
    ) -> Self {
        Self {
            log_entries,
            last_committed_log_entry_id,
            log_current_term,
        }
    }

    pub fn log_entries(&self) -> &Vec<LogEntry> {
        &self.log_entries
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn log_current_term(&self) -> LogTerm {
        self.log_current_term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralLogEntriesAppendResponseMessageInput {
    acknowledged_log_entry_ids: Vec<LogEntryId>,
    missing_log_entry_ids: Vec<LogEntryId>,
}

impl From<GeneralLogEntriesAppendResponseMessageInput> for (Vec<LogEntryId>, Vec<LogEntryId>) {
    fn from(input: GeneralLogEntriesAppendResponseMessageInput) -> Self {
        (
            input.acknowledged_log_entry_ids,
            input.missing_log_entry_ids,
        )
    }
}

impl GeneralLogEntriesAppendResponseMessageInput {
    pub fn new(
        acknowledged_log_entry_ids: Vec<LogEntryId>,
        missing_log_entry_ids: Vec<LogEntryId>,
    ) -> Self {
        Self {
            acknowledged_log_entry_ids,
            missing_log_entry_ids,
        }
    }

    pub fn acknowledged_log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.acknowledged_log_entry_ids
    }

    pub fn missing_log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.missing_log_entry_ids
    }
}
