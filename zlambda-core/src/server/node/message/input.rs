use crate::common::message::{MessageSocketReceiver, MessageSocketSender};
use crate::common::module::ModuleId;
use crate::common::utility::Bytes;
use crate::general::GeneralMessage;
use crate::server::{LogEntry, LogEntryId, LogTerm};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeReplicationMessageInput {
    log_entries: Vec<LogEntry>,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_current_term: LogTerm,
}

impl From<ServerNodeReplicationMessageInput> for (Vec<LogEntry>, Option<LogEntryId>, LogTerm) {
    fn from(input: ServerNodeReplicationMessageInput) -> Self {
        (
            input.log_entries,
            input.last_committed_log_entry_id,
            input.log_current_term,
        )
    }
}

impl ServerNodeReplicationMessageInput {
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
pub struct ServerNodeRegistrationMessageInput {
    general_socket_sender: MessageSocketSender<GeneralMessage>,
    general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_current_term: LogTerm,
}

impl From<ServerNodeRegistrationMessageInput>
    for (
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
        Option<LogEntryId>,
        LogTerm,
    )
{
    fn from(input: ServerNodeRegistrationMessageInput) -> Self {
        (
            input.general_socket_sender,
            input.general_socket_receiver,
            input.last_committed_log_entry_id,
            input.log_current_term,
        )
    }
}

impl ServerNodeRegistrationMessageInput {
    pub fn new(
        general_socket_sender: MessageSocketSender<GeneralMessage>,
        general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_current_term: LogTerm,
    ) -> Self {
        Self {
            general_socket_sender,
            general_socket_receiver,
            last_committed_log_entry_id,
            log_current_term,
        }
    }

    pub fn general_socket_sender(&self) -> &MessageSocketSender<GeneralMessage> {
        &self.general_socket_sender
    }

    pub fn general_socket_receiver(&self) -> &MessageSocketReceiver<GeneralMessage> {
        &self.general_socket_receiver
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
pub struct ServerNodeRecoveryMessageInput {
    general_socket_sender: MessageSocketSender<GeneralMessage>,
    general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_current_term: LogTerm,
}

impl From<ServerNodeRecoveryMessageInput>
    for (
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
        Option<LogEntryId>,
        LogTerm,
    )
{
    fn from(input: ServerNodeRecoveryMessageInput) -> Self {
        (
            input.general_socket_sender,
            input.general_socket_receiver,
            input.last_committed_log_entry_id,
            input.log_current_term,
        )
    }
}

impl ServerNodeRecoveryMessageInput {
    pub fn new(
        general_socket_sender: MessageSocketSender<GeneralMessage>,
        general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_current_term: LogTerm,
    ) -> Self {
        Self {
            general_socket_sender,
            general_socket_receiver,
            last_committed_log_entry_id,
            log_current_term,
        }
    }

    pub fn general_socket_sender(&self) -> &MessageSocketSender<GeneralMessage> {
        &self.general_socket_sender
    }

    pub fn general_socket_receiver(&self) -> &MessageSocketReceiver<GeneralMessage> {
        &self.general_socket_receiver
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
pub struct ServerNodeNotifyMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ServerNodeNotifyMessageInput> for (ModuleId, Bytes) {
    fn from(input: ServerNodeNotifyMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ServerNodeNotifyMessageInput {
    pub fn new(module_id: ModuleId, body: Bytes) -> Self {
        Self { module_id, body }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeNotificationImmediateMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ServerNodeNotificationImmediateMessageInput> for (ModuleId, Bytes) {
    fn from(input: ServerNodeNotificationImmediateMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ServerNodeNotificationImmediateMessageInput {
    pub fn new(module_id: ModuleId, body: Bytes) -> Self {
        Self { module_id, body }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeNotificationStartMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ServerNodeNotificationStartMessageInput> for (ModuleId, Bytes) {
    fn from(input: ServerNodeNotificationStartMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ServerNodeNotificationStartMessageInput {
    pub fn new(module_id: ModuleId, body: Bytes) -> Self {
        Self { module_id, body }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeNotificationNextMessageInput {
    notification_id: usize,
    body: Bytes,
}

impl From<ServerNodeNotificationNextMessageInput> for (usize, Bytes) {
    fn from(input: ServerNodeNotificationNextMessageInput) -> Self {
        (input.notification_id, input.body)
    }
}

impl ServerNodeNotificationNextMessageInput {
    pub fn new(notification_id: usize, body: Bytes) -> Self {
        Self {
            notification_id,
            body,
        }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeNotificationEndMessageInput {
    notification_id: usize,
    body: Bytes,
}

impl From<ServerNodeNotificationEndMessageInput> for (usize, Bytes) {
    fn from(input: ServerNodeNotificationEndMessageInput) -> Self {
        (input.notification_id, input.body)
    }
}

impl ServerNodeNotificationEndMessageInput {
    pub fn new(notification_id: usize, body: Bytes) -> Self {
        Self {
            notification_id,
            body,
        }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}
