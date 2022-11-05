use crate::cluster::LogEntry;
use actix::Message;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ReplicateLogEntryActorMessage {
    log_entry: LogEntry,
}

impl From<ReplicateLogEntryActorMessage> for (LogEntry,) {
    fn from(message: ReplicateLogEntryActorMessage) -> Self {
        (message.log_entry,)
    }
}

impl Message for ReplicateLogEntryActorMessage {
    type Result = ();
}

impl ReplicateLogEntryActorMessage {
    pub fn new(log_entry: LogEntry) -> Self {
        Self { log_entry }
    }

    pub fn log_entry(&self) -> &LogEntry {
        &self.log_entry
    }
}
