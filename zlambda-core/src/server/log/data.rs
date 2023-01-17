use crate::server::ServerId;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub enum LogEntryData {
    ServerSocketAddresses(Vec<Option<ServerId>>),
}
