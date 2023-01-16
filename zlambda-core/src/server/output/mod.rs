use crate::server::{LogTerm, ServerId};
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
    log_term: LogTerm,
}

impl From<ServerRegistrationMessageSuccessOutput>
    for (ServerId, ServerId, Vec<Option<SocketAddr>>, LogTerm)
{
    fn from(output: ServerRegistrationMessageSuccessOutput) -> Self {
        (
            output.server_id,
            output.leader_server_id,
            output.server_socket_addresses,
            output.log_term,
        )
    }
}

impl ServerRegistrationMessageSuccessOutput {
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
    log_term: LogTerm,
}

impl From<ServerRecoveryMessageSuccessOutput>
    for (ServerId, Vec<Option<SocketAddr>>, LogTerm)
{
    fn from(output: ServerRecoveryMessageSuccessOutput) -> Self {
        (
            output.leader_server_id,
            output.server_socket_addresses,
            output.log_term,
        )
    }
}

impl ServerRecoveryMessageSuccessOutput {
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

#[derive(Debug)]
pub enum ServerRecoveryMessageOutput {
    NotALeader(ServerRecoveryMessageNotALeaderOutput),
    Success(ServerRecoveryMessageSuccessOutput),
    IsOnline,
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
