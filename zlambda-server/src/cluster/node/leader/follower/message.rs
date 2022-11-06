use crate::cluster::{LogEntryId, LogEntryType};
use actix::Message;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ReplicateLogEntryActorMessage {
    log_entry_id: LogEntryId,
    log_entry_type: LogEntryType,
    last_committed_log_entry_id: Option<LogEntryId>,
}

impl From<ReplicateLogEntryActorMessage> for (LogEntryId, LogEntryType, Option<LogEntryId>) {
    fn from(message: ReplicateLogEntryActorMessage) -> Self {
        (
            message.log_entry_id,
            message.log_entry_type,
            message.last_committed_log_entry_id,
        )
    }
}

impl Message for ReplicateLogEntryActorMessage {
    type Result = ();
}

impl ReplicateLogEntryActorMessage {
    pub fn new(
        log_entry_id: LogEntryId,
        log_entry_type: LogEntryType,
        last_committed_log_entry_id: Option<LogEntryId>,
    ) -> Self {
        Self {
            log_entry_id,
            log_entry_type,
            last_committed_log_entry_id,
        }
    }

    pub fn log_entry_id(&self) -> LogEntryId {
        self.log_entry_id
    }

    pub fn log_entry_type(&self) -> &LogEntryType {
        &self.log_entry_type
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct SendLogEntryRequestActorMessage;

impl Message for SendLogEntryRequestActorMessage {
    type Result = ();
}
