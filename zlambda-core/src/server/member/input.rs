use crate::server::LogEntry;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerMemberReplicateMessageInput {
    log_entries: Vec<LogEntry>,
}

impl From<ServerMemberReplicateMessageInput> for (Vec<LogEntry>,) {
    fn from(input: ServerMemberReplicateMessageInput) -> Self {
        (input.log_entries,)
    }
}

impl ServerMemberReplicateMessageInput {
    pub fn new(log_entries: Vec<LogEntry>) -> Self {
        Self { log_entries }
    }

    pub fn log_entries(&self) -> &Vec<LogEntry> {
        &self.log_entries
    }
}
