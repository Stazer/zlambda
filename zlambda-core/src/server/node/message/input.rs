use crate::common::message::{MessageSocketReceiver, MessageSocketSender};
use crate::common::module::ModuleId;
use crate::common::utility::Bytes;
use crate::general::GeneralMessage;
use crate::server::{
    LogEntry, LogEntryId, LogEntryIssueId, LogId, LogTerm, ServerClientId, ServerId,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeShutdownMessageInput {}

impl ServerNodeShutdownMessageInput {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeReplicationMessageInput {
    log_id: LogId,
    log_entries: Vec<LogEntry>,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_current_term: LogTerm,
}

impl From<ServerNodeReplicationMessageInput>
    for (LogId, Vec<LogEntry>, Option<LogEntryId>, LogTerm)
{
    fn from(input: ServerNodeReplicationMessageInput) -> Self {
        (
            input.log_id,
            input.log_entries,
            input.last_committed_log_entry_id,
            input.log_current_term,
        )
    }
}

impl ServerNodeReplicationMessageInput {
    pub fn new(
        log_id: LogId,
        log_entries: Vec<LogEntry>,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_current_term: LogTerm,
    ) -> Self {
        Self {
            log_id,
            log_entries,
            last_committed_log_entry_id,
            log_current_term,
        }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
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
pub struct ServerNodeNodeHandshakeMessageInput {
    general_message_sender: MessageSocketSender<GeneralMessage>,
    general_message_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl From<ServerNodeNodeHandshakeMessageInput>
    for (
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
    )
{
    fn from(input: ServerNodeNodeHandshakeMessageInput) -> Self {
        (input.general_message_sender, input.general_message_receiver)
    }
}

impl ServerNodeNodeHandshakeMessageInput {
    pub fn new(
        general_message_sender: MessageSocketSender<GeneralMessage>,
        general_message_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            general_message_sender,
            general_message_receiver,
        }
    }

    pub fn general_message_sender(&self) -> &MessageSocketSender<GeneralMessage> {
        &self.general_message_sender
    }

    pub fn general_message_receiver(&self) -> &MessageSocketReceiver<GeneralMessage> {
        &self.general_message_receiver
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeLogAppendResponseMessageInput {
    log_id: LogId,
    log_entry_ids: Vec<LogEntryId>,
    missing_log_entry_ids: Vec<LogEntryId>,
}

impl From<ServerNodeLogAppendResponseMessageInput> for (LogId, Vec<LogEntryId>, Vec<LogEntryId>) {
    fn from(input: ServerNodeLogAppendResponseMessageInput) -> Self {
        (
            input.log_id,
            input.log_entry_ids,
            input.missing_log_entry_ids,
        )
    }
}

impl ServerNodeLogAppendResponseMessageInput {
    pub fn new(
        log_id: LogId,
        log_entry_ids: Vec<LogEntryId>,
        missing_log_entry_ids: Vec<LogEntryId>,
    ) -> Self {
        Self {
            log_id,
            log_entry_ids,
            missing_log_entry_ids,
        }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.log_entry_ids
    }

    pub fn missing_log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.missing_log_entry_ids
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeLogAppendInitiateMessageInput {
    log_id: LogId,
}

impl From<ServerNodeLogAppendInitiateMessageInput> for (LogId,) {
    fn from(input: ServerNodeLogAppendInitiateMessageInput) -> Self {
        (input.log_id,)
    }
}

impl ServerNodeLogAppendInitiateMessageInput {
    pub fn new(log_id: LogId) -> Self {
        Self { log_id }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeLogEntriesCommitMessageInput {
    log_id: LogId,
    log_entry_data: Bytes,
    log_entry_issue_id: LogEntryIssueId,
}

impl From<ServerNodeLogEntriesCommitMessageInput> for (LogId, Bytes, LogEntryIssueId) {
    fn from(input: ServerNodeLogEntriesCommitMessageInput) -> Self {
        (input.log_id, input.log_entry_data, input.log_entry_issue_id)
    }
}

impl ServerNodeLogEntriesCommitMessageInput {
    pub fn new(log_id: LogId, log_entry_data: Bytes, log_entry_issue_id: LogEntryIssueId) -> Self {
        Self {
            log_id,
            log_entry_data,
            log_entry_issue_id,
        }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn log_entry_data(&self) -> &Bytes {
        &self.log_entry_data
    }

    pub fn log_entry_issue_id(&self) -> LogEntryIssueId {
        self.log_entry_issue_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeNotificationOriginInput {
    server_id: ServerId,
    server_client_id: ServerClientId,
}

impl ServerNodeNotificationOriginInput {
    pub fn new(server_id: ServerId, server_client_id: ServerClientId) -> Self {
        Self {
            server_id,
            server_client_id,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn server_client_id(&self) -> ServerClientId {
        self.server_client_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeNotificationImmediateMessageInput {
    module_id: ModuleId,
    body: Bytes,
    origin: Option<ServerNodeNotificationOriginInput>,
}

impl From<ServerNodeNotificationImmediateMessageInput> for (ModuleId, Bytes) {
    fn from(input: ServerNodeNotificationImmediateMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl From<ServerNodeNotificationImmediateMessageInput>
    for (ModuleId, Bytes, Option<ServerNodeNotificationOriginInput>)
{
    fn from(input: ServerNodeNotificationImmediateMessageInput) -> Self {
        (input.module_id, input.body, input.origin)
    }
}

impl ServerNodeNotificationImmediateMessageInput {
    pub fn new(
        module_id: ModuleId,
        body: Bytes,
        origin: Option<ServerNodeNotificationOriginInput>,
    ) -> Self {
        Self {
            module_id,
            body,
            origin,
        }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }

    pub fn origin(&self) -> &Option<ServerNodeNotificationOriginInput> {
        &self.origin
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerNodeNotificationStartMessageInput {
    module_id: ModuleId,
    body: Bytes,
    origin: Option<ServerNodeNotificationOriginInput>,
}

impl From<ServerNodeNotificationStartMessageInput> for (ModuleId, Bytes) {
    fn from(input: ServerNodeNotificationStartMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl From<ServerNodeNotificationStartMessageInput>
    for (ModuleId, Bytes, Option<ServerNodeNotificationOriginInput>)
{
    fn from(input: ServerNodeNotificationStartMessageInput) -> Self {
        (input.module_id, input.body, input.origin)
    }
}

impl ServerNodeNotificationStartMessageInput {
    pub fn new(
        module_id: ModuleId,
        body: Bytes,
        origin: Option<ServerNodeNotificationOriginInput>,
    ) -> Self {
        Self {
            module_id,
            body,
            origin,
        }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }

    pub fn origin(&self) -> &Option<ServerNodeNotificationOriginInput> {
        &self.origin
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
