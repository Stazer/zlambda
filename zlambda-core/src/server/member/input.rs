use crate::general::GeneralMessage;
use crate::message::{MessageSocketReceiver, MessageSocketSender};
use crate::server::{LogEntry, LogEntryId, LogTerm};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerMemberReplicationMessageInput {
    log_entries: Vec<LogEntry>,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_current_term: LogTerm,
}

impl From<ServerMemberReplicationMessageInput> for (Vec<LogEntry>, Option<LogEntryId>, LogTerm) {
    fn from(input: ServerMemberReplicationMessageInput) -> Self {
        (
            input.log_entries,
            input.last_committed_log_entry_id,
            input.log_current_term,
        )
    }
}

impl ServerMemberReplicationMessageInput {
    pub fn new(
        log_entries: Vec<LogEntry>,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_current_term: LogTerm,
    ) -> Self {
        Self {
            log_entries,
            last_committed_log_entry_id,
            log_current_term,
        }
    }

    pub fn log_entries(&self) -> &Vec<LogEntry> {
        &self.log_entries
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn log_current_term(&self) -> LogTerm {
        self.log_current_term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerMemberRegistrationMessageInput {
    general_socket_sender: MessageSocketSender<GeneralMessage>,
    general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl From<ServerMemberRegistrationMessageInput>
    for (
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
    )
{
    fn from(input: ServerMemberRegistrationMessageInput) -> Self {
        (input.general_socket_sender, input.general_socket_receiver)
    }
}

impl ServerMemberRegistrationMessageInput {
    pub fn new(
        general_socket_sender: MessageSocketSender<GeneralMessage>,
        general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            general_socket_sender,
            general_socket_receiver,
        }
    }

    pub fn general_socket_sender(&self) -> &MessageSocketSender<GeneralMessage> {
        &self.general_socket_sender
    }

    pub fn general_socket_receiver(&self) -> &MessageSocketReceiver<GeneralMessage> {
        &self.general_socket_receiver
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerMemberRecoveryMessageInput {
    general_socket_sender: MessageSocketSender<GeneralMessage>,
    general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl From<ServerMemberRecoveryMessageInput>
    for (
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
    )
{
    fn from(input: ServerMemberRecoveryMessageInput) -> Self {
        (input.general_socket_sender, input.general_socket_receiver)
    }
}

impl ServerMemberRecoveryMessageInput {
    pub fn new(
        general_socket_sender: MessageSocketSender<GeneralMessage>,
        general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            general_socket_sender,
            general_socket_receiver,
        }
    }

    pub fn general_socket_sender(&self) -> &MessageSocketSender<GeneralMessage> {
        &self.general_socket_sender
    }

    pub fn general_socket_receiver(&self) -> &MessageSocketReceiver<GeneralMessage> {
        &self.general_socket_receiver
    }
}
