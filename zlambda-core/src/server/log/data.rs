use crate::server::ServerId;
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
pub enum LogEntryData {
    AddServer(AddServerLogEntryData),
    RemoveServer(RemoveServerLogEntryData),
}
