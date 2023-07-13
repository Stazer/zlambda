use crate::server::{LogId, LogIssuer, ServerId};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct AddServerLogEntryData {
    server_id: ServerId,
    server_socket_address: SocketAddr,
}

impl From<AddServerLogEntryData> for (ServerId, SocketAddr) {
    fn from(data: AddServerLogEntryData) -> Self {
        (data.server_id, data.server_socket_address)
    }
}

impl AddServerLogEntryData {
    pub fn new(server_id: ServerId, server_socket_address: SocketAddr) -> Self {
        Self {
            server_id,
            server_socket_address,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn server_socket_address(&self) -> SocketAddr {
        self.server_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct RemoveServerLogEntryData {
    server_id: ServerId,
}

impl From<RemoveServerLogEntryData> for (ServerId,) {
    fn from(data: RemoveServerLogEntryData) -> Self {
        (data.server_id,)
    }
}

impl RemoveServerLogEntryData {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct ServerSystemCreateLogLogEntryData {
    log_id: LogId,
    log_issuer: Option<LogIssuer>,
}

impl From<ServerSystemCreateLogLogEntryData> for (LogId, Option<LogIssuer>) {
    fn from(data: ServerSystemCreateLogLogEntryData) -> Self {
        (data.log_id, data.log_issuer)
    }
}

impl ServerSystemCreateLogLogEntryData {
    pub fn new(log_id: LogId, log_issuer: Option<LogIssuer>) -> Self {
        Self { log_id, log_issuer }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn log_issuer(&self) -> &Option<LogIssuer> {
        &self.log_issuer
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub enum ServerSystemLogEntryData {
    AddServer(AddServerLogEntryData),
    RemoveServer(RemoveServerLogEntryData),
    CreateLog(ServerSystemCreateLogLogEntryData),
}

impl From<AddServerLogEntryData> for ServerSystemLogEntryData {
    fn from(data: AddServerLogEntryData) -> Self {
        Self::AddServer(data)
    }
}

impl From<RemoveServerLogEntryData> for ServerSystemLogEntryData {
    fn from(data: RemoveServerLogEntryData) -> Self {
        Self::RemoveServer(data)
    }
}

impl From<ServerSystemCreateLogLogEntryData> for ServerSystemLogEntryData {
    fn from(data: ServerSystemCreateLogLogEntryData) -> Self {
        Self::CreateLog(data)
    }
}
