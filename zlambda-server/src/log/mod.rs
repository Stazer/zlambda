pub mod following;
pub mod leading;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::node::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogEntryId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClusterLogEntryType {
    Addresses(HashMap<NodeId, SocketAddr>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ModuleLogEntryType {
    Initialize,
    Append(u64, Vec<u8>),
    Load(u64),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientLogEntryType {
    Module(ModuleLogEntryType),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogEntryType {
    Cluster(ClusterLogEntryType),
    Client(ClientLogEntryType),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogEntryData {
    id: LogEntryId,
    r#type: LogEntryType,
}

impl LogEntryData {
    pub fn new(id: LogEntryId, r#type: LogEntryType) -> Self {
        Self { id, r#type }
    }

    pub fn id(&self) -> LogEntryId {
        self.id
    }

    pub fn r#type(&self) -> &LogEntryType {
        &self.r#type
    }
}
