use crate::server::LogTerm;
use crate::server::ServerId;
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
pub struct GeneralLogEntriesAppendRequestInput {}

impl GeneralLogEntriesAppendRequestInput {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralLogEntriesAppendResponseInput {}

impl GeneralLogEntriesAppendResponseInput {}
