use crate::common::module::ModuleId;
use crate::common::utility::Bytes;
use crate::server::{
    LogEntry, LogEntryId, LogEntryIssueId, LogId, LogTerm, ServerClientId, ServerId,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRegistrationRequestMessageInput {
    server_socket_address: SocketAddr,
}

impl From<GeneralRegistrationRequestMessageInput> for (SocketAddr,) {
    fn from(input: GeneralRegistrationRequestMessageInput) -> Self {
        (input.server_socket_address,)
    }
}

impl GeneralRegistrationRequestMessageInput {
    pub fn new(server_socket_address: SocketAddr) -> Self {
        Self {
            server_socket_address,
        }
    }

    pub fn server_socket_address(&self) -> &SocketAddr {
        &self.server_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRegistrationResponseMessageNotALeaderInput {
    leader_server_socket_address: SocketAddr,
}

impl From<GeneralRegistrationResponseMessageNotALeaderInput> for (SocketAddr,) {
    fn from(input: GeneralRegistrationResponseMessageNotALeaderInput) -> Self {
        (input.leader_server_socket_address,)
    }
}

impl GeneralRegistrationResponseMessageNotALeaderInput {
    pub fn new(leader_server_socket_address: SocketAddr) -> Self {
        Self {
            leader_server_socket_address,
        }
    }

    pub fn leader_server_socket_address(&self) -> &SocketAddr {
        &self.leader_server_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRegistrationResponseMessageSuccessInput {
    server_id: ServerId,
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    log_term: LogTerm,
}

impl From<GeneralRegistrationResponseMessageSuccessInput>
    for (ServerId, ServerId, Vec<Option<SocketAddr>>, LogTerm)
{
    fn from(input: GeneralRegistrationResponseMessageSuccessInput) -> Self {
        (
            input.server_id,
            input.leader_server_id,
            input.server_socket_addresses,
            input.log_term,
        )
    }
}

impl GeneralRegistrationResponseMessageSuccessInput {
    pub fn new(
        server_id: ServerId,
        leader_server_id: ServerId,
        server_socket_addresses: Vec<Option<SocketAddr>>,
        log_term: LogTerm,
    ) -> Self {
        Self {
            server_id,
            leader_server_id,
            server_socket_addresses,
            log_term,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn leader_server_id(&self) -> ServerId {
        self.leader_server_id
    }

    pub fn server_socket_addresses(&self) -> &Vec<Option<SocketAddr>> {
        &self.server_socket_addresses
    }

    pub fn log_term(&self) -> LogTerm {
        self.log_term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralRegistrationResponseMessageInput {
    NotALeader(GeneralRegistrationResponseMessageNotALeaderInput),
    Success(GeneralRegistrationResponseMessageSuccessInput),
}

impl From<GeneralRegistrationResponseMessageNotALeaderInput>
    for GeneralRegistrationResponseMessageInput
{
    fn from(input: GeneralRegistrationResponseMessageNotALeaderInput) -> Self {
        Self::NotALeader(input)
    }
}

impl From<GeneralRegistrationResponseMessageSuccessInput>
    for GeneralRegistrationResponseMessageInput
{
    fn from(input: GeneralRegistrationResponseMessageSuccessInput) -> Self {
        Self::Success(input)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRecoveryRequestMessageInput {
    server_id: ServerId,
}

impl From<GeneralRecoveryRequestMessageInput> for (ServerId,) {
    fn from(input: GeneralRecoveryRequestMessageInput) -> Self {
        (input.server_id,)
    }
}

impl GeneralRecoveryRequestMessageInput {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> &ServerId {
        &self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRecoveryResponseMessageNotALeaderInput {
    leader_socket_address: SocketAddr,
}

impl From<GeneralRecoveryResponseMessageNotALeaderInput> for (SocketAddr,) {
    fn from(input: GeneralRecoveryResponseMessageNotALeaderInput) -> Self {
        (input.leader_socket_address,)
    }
}

impl GeneralRecoveryResponseMessageNotALeaderInput {
    pub fn new(leader_socket_address: SocketAddr) -> Self {
        Self {
            leader_socket_address,
        }
    }

    pub fn leader_socket_address(&self) -> &SocketAddr {
        &self.leader_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralRecoveryResponseMessageSuccessInput {
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    log_term: LogTerm,
}

impl From<GeneralRecoveryResponseMessageSuccessInput>
    for (ServerId, Vec<Option<SocketAddr>>, LogTerm)
{
    fn from(input: GeneralRecoveryResponseMessageSuccessInput) -> Self {
        (
            input.leader_server_id,
            input.server_socket_addresses,
            input.log_term,
        )
    }
}

impl GeneralRecoveryResponseMessageSuccessInput {
    pub fn new(
        leader_server_id: ServerId,
        server_socket_addresses: Vec<Option<SocketAddr>>,
        log_term: LogTerm,
    ) -> Self {
        Self {
            leader_server_id,
            server_socket_addresses,
            log_term,
        }
    }

    pub fn leader_server_id(&self) -> ServerId {
        self.leader_server_id
    }

    pub fn server_socket_addresses(&self) -> &Vec<Option<SocketAddr>> {
        &self.server_socket_addresses
    }

    pub fn log_term(&self) -> LogTerm {
        self.log_term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralRecoveryResponseMessageInput {
    NotALeader(GeneralRecoveryResponseMessageNotALeaderInput),
    IsOnline,
    Unknown,
    Success(GeneralRecoveryResponseMessageSuccessInput),
}

impl From<GeneralRecoveryResponseMessageNotALeaderInput> for GeneralRecoveryResponseMessageInput {
    fn from(input: GeneralRecoveryResponseMessageNotALeaderInput) -> Self {
        Self::NotALeader(input)
    }
}

impl From<GeneralRecoveryResponseMessageSuccessInput> for GeneralRecoveryResponseMessageInput {
    fn from(input: GeneralRecoveryResponseMessageSuccessInput) -> Self {
        Self::Success(input)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNodeHandshakeRequestMessageInput {
    server_id: ServerId,
}

impl From<GeneralNodeHandshakeRequestMessageInput> for (ServerId,) {
    fn from(input: GeneralNodeHandshakeRequestMessageInput) -> Self {
        (input.server_id,)
    }
}

impl GeneralNodeHandshakeRequestMessageInput {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralNodeHandshakeResponseMessageInputResult {
    ServerIdUnfeasible,
    AlreadyOnline,
    Unknown,
    Success,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNodeHandshakeResponseMessageInput {
    result: GeneralNodeHandshakeResponseMessageInputResult,
}

impl From<GeneralNodeHandshakeResponseMessageInput>
    for (GeneralNodeHandshakeResponseMessageInputResult,)
{
    fn from(input: GeneralNodeHandshakeResponseMessageInput) -> Self {
        (input.result,)
    }
}

impl GeneralNodeHandshakeResponseMessageInput {
    pub fn new(result: GeneralNodeHandshakeResponseMessageInputResult) -> Self {
        Self { result }
    }

    pub fn result(&self) -> &GeneralNodeHandshakeResponseMessageInputResult {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralLogEntriesAppendRequestMessageInput {
    log_id: LogId,
    log_entries: Vec<LogEntry>,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_current_term: LogTerm,
}

impl From<GeneralLogEntriesAppendRequestMessageInput>
    for (LogId, Vec<LogEntry>, Option<LogEntryId>, LogTerm)
{
    fn from(input: GeneralLogEntriesAppendRequestMessageInput) -> Self {
        (
            input.log_id,
            input.log_entries,
            input.last_committed_log_entry_id,
            input.log_current_term,
        )
    }
}

impl GeneralLogEntriesAppendRequestMessageInput {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralLogEntriesAppendResponseMessageInput {
    log_id: LogId,
    acknowledged_log_entry_ids: Vec<LogEntryId>,
    missing_log_entry_ids: Vec<LogEntryId>,
}

impl From<GeneralLogEntriesAppendResponseMessageInput>
    for (LogId, Vec<LogEntryId>, Vec<LogEntryId>)
{
    fn from(input: GeneralLogEntriesAppendResponseMessageInput) -> Self {
        (
            input.log_id,
            input.acknowledged_log_entry_ids,
            input.missing_log_entry_ids,
        )
    }
}

impl GeneralLogEntriesAppendResponseMessageInput {
    pub fn new(
        log_id: LogId,
        acknowledged_log_entry_ids: Vec<LogEntryId>,
        missing_log_entry_ids: Vec<LogEntryId>,
    ) -> Self {
        Self {
            log_id,
            acknowledged_log_entry_ids,
            missing_log_entry_ids,
        }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn acknowledged_log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.acknowledged_log_entry_ids
    }

    pub fn missing_log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.missing_log_entry_ids
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralLogEntriesAppendInitiateMessageInput {
    log_id: LogId,
}

impl From<GeneralLogEntriesAppendInitiateMessageInput> for (LogId,) {
    fn from(input: GeneralLogEntriesAppendInitiateMessageInput) -> Self {
        (input.log_id,)
    }
}

impl GeneralLogEntriesAppendInitiateMessageInput {
    pub fn new(log_id: LogId) -> Self {
        Self { log_id }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralLogEntriesCommitMessageInput {
    log_id: LogId,
    log_entry_data: Bytes,
    log_entry_issue_id: LogEntryIssueId,
}

impl From<GeneralLogEntriesCommitMessageInput> for (LogId, Bytes, LogEntryIssueId) {
    fn from(input: GeneralLogEntriesCommitMessageInput) -> Self {
        (input.log_id, input.log_entry_data, input.log_entry_issue_id)
    }
}

impl GeneralLogEntriesCommitMessageInput {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralClientRegistrationRequestMessageInput;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralClientRegistrationResponseMessageInput;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNotificationMessageInputOrigin {
    server_id: ServerId,
    server_client_id: ServerClientId,
}

impl GeneralNotificationMessageInputOrigin {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNotificationMessageInputRedirection {
    server_id: ServerId,
    server_client_id: Option<ServerClientId>,
}

impl GeneralNotificationMessageInputRedirection {
    pub fn new(server_id: ServerId, server_client_id: Option<ServerClientId>) -> Self {
        Self {
            server_id,
            server_client_id,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn server_client_id(&self) -> Option<ServerClientId> {
        self.server_client_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNotificationMessageInputImmediateType {
    module_id: ModuleId,
    origin: Option<GeneralNotificationMessageInputOrigin>,
    redirection: Option<GeneralNotificationMessageInputRedirection>,
}

impl From<GeneralNotificationMessageInputImmediateType> for (ModuleId,) {
    fn from(r#type: GeneralNotificationMessageInputImmediateType) -> Self {
        (r#type.module_id,)
    }
}

impl From<GeneralNotificationMessageInputImmediateType>
    for (ModuleId, Option<GeneralNotificationMessageInputOrigin>)
{
    fn from(r#type: GeneralNotificationMessageInputImmediateType) -> Self {
        (r#type.module_id, r#type.origin)
    }
}

impl GeneralNotificationMessageInputImmediateType {
    pub fn new(
        module_id: ModuleId,
        origin: Option<GeneralNotificationMessageInputOrigin>,
        redirection: Option<GeneralNotificationMessageInputRedirection>,
    ) -> Self {
        Self {
            module_id,
            origin,
            redirection,
        }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn origin(&self) -> &Option<GeneralNotificationMessageInputOrigin> {
        &self.origin
    }

    pub fn redirection(&self) -> &Option<GeneralNotificationMessageInputRedirection> {
        &self.redirection
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNotificationMessageInputStartType {
    module_id: ModuleId,
    notification_id: usize,
    origin: Option<GeneralNotificationMessageInputOrigin>,
    redirection: Option<GeneralNotificationMessageInputRedirection>,
}

impl From<GeneralNotificationMessageInputStartType> for (ModuleId, usize) {
    fn from(r#type: GeneralNotificationMessageInputStartType) -> Self {
        (r#type.module_id, r#type.notification_id)
    }
}

impl From<GeneralNotificationMessageInputStartType>
    for (
        ModuleId,
        usize,
        Option<GeneralNotificationMessageInputOrigin>,
    )
{
    fn from(r#type: GeneralNotificationMessageInputStartType) -> Self {
        (r#type.module_id, r#type.notification_id, r#type.origin)
    }
}

impl GeneralNotificationMessageInputStartType {
    pub fn new(
        module_id: ModuleId,
        notification_id: usize,
        origin: Option<GeneralNotificationMessageInputOrigin>,
        redirection: Option<GeneralNotificationMessageInputRedirection>,
    ) -> Self {
        Self {
            module_id,
            notification_id,
            origin,
            redirection,
        }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }

    pub fn origin(&self) -> &Option<GeneralNotificationMessageInputOrigin> {
        &self.origin
    }

    pub fn redirection(&self) -> &Option<GeneralNotificationMessageInputRedirection> {
        &self.redirection
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNotificationMessageInputNextType {
    notification_id: usize,
}

impl From<GeneralNotificationMessageInputNextType> for (usize,) {
    fn from(r#type: GeneralNotificationMessageInputNextType) -> Self {
        (r#type.notification_id,)
    }
}

impl GeneralNotificationMessageInputNextType {
    pub fn new(notification_id: usize) -> Self {
        Self { notification_id }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNotificationMessageInputEndType {
    notification_id: usize,
}

impl From<GeneralNotificationMessageInputEndType> for (usize,) {
    fn from(r#type: GeneralNotificationMessageInputEndType) -> Self {
        (r#type.notification_id,)
    }
}

impl GeneralNotificationMessageInputEndType {
    pub fn new(notification_id: usize) -> Self {
        Self { notification_id }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GeneralNotificationMessageInputType {
    Immediate(GeneralNotificationMessageInputImmediateType),
    Start(GeneralNotificationMessageInputStartType),
    Next(GeneralNotificationMessageInputNextType),
    End(GeneralNotificationMessageInputEndType),
}

impl From<GeneralNotificationMessageInputImmediateType> for GeneralNotificationMessageInputType {
    fn from(r#type: GeneralNotificationMessageInputImmediateType) -> Self {
        Self::Immediate(r#type)
    }
}

impl From<GeneralNotificationMessageInputStartType> for GeneralNotificationMessageInputType {
    fn from(r#type: GeneralNotificationMessageInputStartType) -> Self {
        Self::Start(r#type)
    }
}

impl From<GeneralNotificationMessageInputNextType> for GeneralNotificationMessageInputType {
    fn from(r#type: GeneralNotificationMessageInputNextType) -> Self {
        Self::Next(r#type)
    }
}

impl From<GeneralNotificationMessageInputEndType> for GeneralNotificationMessageInputType {
    fn from(r#type: GeneralNotificationMessageInputEndType) -> Self {
        Self::End(r#type)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralNotificationMessageInput {
    r#type: GeneralNotificationMessageInputType,
    body: Bytes,
}

impl From<GeneralNotificationMessageInput> for (GeneralNotificationMessageInputType, Bytes) {
    fn from(input: GeneralNotificationMessageInput) -> Self {
        (input.r#type, input.body)
    }
}

impl GeneralNotificationMessageInput {
    pub fn new(r#type: GeneralNotificationMessageInputType, body: Bytes) -> Self {
        Self { r#type, body }
    }

    pub fn r#type(&self) -> &GeneralNotificationMessageInputType {
        &self.r#type
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneralClientRedirectMessageInput {
    server_id: ServerId,
    server_client_id: ServerClientId,
    body: Bytes,
}

impl From<GeneralClientRedirectMessageInput> for (ServerId, ServerClientId, Bytes) {
    fn from(input: GeneralClientRedirectMessageInput) -> Self {
        (input.server_id, input.server_client_id, input.body)
    }
}

impl GeneralClientRedirectMessageInput {
    pub fn new(server_id: ServerId, server_client_id: ServerClientId, body: Bytes) -> Self {
        Self {
            server_id,
            server_client_id,
            body,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn server_client_id(&self) -> ServerClientId {
        self.server_client_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}
