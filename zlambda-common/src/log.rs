use crate::module::ModuleId;
use crate::node::NodeId;
use crate::term::Term;
use bytes::Bytes;
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
pub enum ClientLogEntryType {
    Initialize,
    Append(u64, Bytes),
    Insert {
        module_id: ModuleId,
        index: u64,
        bytes: Bytes,
    },
    Load(u64),
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
    term: Term,
}

impl LogEntryData {
    pub fn new(id: LogEntryId, r#type: LogEntryType, term: Term) -> Self {
        Self { id, r#type, term }
    }

    pub fn id(&self) -> LogEntryId {
        self.id
    }

    pub fn r#type(&self) -> &LogEntryType {
        &self.r#type
    }

    pub fn term(&self) -> Term {
        self.term
    }
}
