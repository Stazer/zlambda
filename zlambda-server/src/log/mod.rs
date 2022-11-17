pub mod following;
pub mod leading;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogEntryId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LogEntryType {
    Add(u64),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
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
