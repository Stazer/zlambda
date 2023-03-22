use crate::common::message::{
    MessageSocketReceiver, MessageSocketSender, SynchronizableMessageOutputSender,
    SynchronousMessageOutputSender,
};
use crate::common::module::ModuleId;
use crate::common::net::TcpStream;
use crate::common::utility::Bytes;
use crate::general::GeneralMessage;
use crate::server::client::ServerClientId;
use crate::server::{
    LogEntry, LogEntryData, LogEntryId, LogId, LogIssuer, LogTerm, ServerId,
    ServerLogCreateMessageOutput, ServerModule,
};
use std::net::SocketAddr;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerSocketAcceptMessageInput {
    socket_stream: TcpStream,
    socket_address: SocketAddr,
}

impl From<ServerSocketAcceptMessageInput> for (TcpStream, SocketAddr) {
    fn from(input: ServerSocketAcceptMessageInput) -> Self {
        (input.socket_stream, input.socket_address)
    }
}

impl ServerSocketAcceptMessageInput {
    pub fn new(socket_stream: TcpStream, socket_address: SocketAddr) -> Self {
        Self {
            socket_stream,
            socket_address,
        }
    }

    pub fn socket_stream(&self) -> &TcpStream {
        &self.socket_stream
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRegistrationMessageInput {
    server_socket_address: SocketAddr,
}

impl From<ServerRegistrationMessageInput> for (SocketAddr,) {
    fn from(input: ServerRegistrationMessageInput) -> Self {
        (input.server_socket_address,)
    }
}

impl ServerRegistrationMessageInput {
    pub fn new(server_socket_address: SocketAddr) -> Self {
        Self {
            server_socket_address,
        }
    }

    pub fn server_socket_address(&self) -> SocketAddr {
        self.server_socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerCommitRegistrationMessageInput {
    node_server_id: ServerId,
}

impl From<ServerCommitRegistrationMessageInput> for (ServerId,) {
    fn from(input: ServerCommitRegistrationMessageInput) -> Self {
        (input.node_server_id,)
    }
}

impl ServerCommitRegistrationMessageInput {
    pub fn new(node_server_id: ServerId) -> Self {
        Self { node_server_id }
    }

    pub fn node_server_id(&self) -> ServerId {
        self.node_server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRecoveryMessageInput {
    server_id: ServerId,
}

impl From<ServerRecoveryMessageInput> for (ServerId,) {
    fn from(input: ServerRecoveryMessageInput) -> Self {
        (input.server_id,)
    }
}

impl ServerRecoveryMessageInput {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogEntriesReplicationMessageInput {
    log_entries_data: Vec<LogEntryData>,
}

impl From<ServerLogEntriesReplicationMessageInput> for (Vec<LogEntryData>,) {
    fn from(input: ServerLogEntriesReplicationMessageInput) -> Self {
        (input.log_entries_data,)
    }
}

impl ServerLogEntriesReplicationMessageInput {
    pub fn new(log_entries_data: Vec<LogEntryData>) -> Self {
        Self { log_entries_data }
    }

    pub fn log_entries_data(&self) -> &Vec<LogEntryData> {
        &self.log_entries_data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogEntriesAcknowledgementMessageInput {
    log_id: LogId,
    log_entry_ids: Vec<LogEntryId>,
    server_id: ServerId,
}

impl From<ServerLogEntriesAcknowledgementMessageInput> for (LogId, Vec<LogEntryId>, ServerId) {
    fn from(input: ServerLogEntriesAcknowledgementMessageInput) -> Self {
        (input.log_id, input.log_entry_ids, input.server_id)
    }
}

impl ServerLogEntriesAcknowledgementMessageInput {
    pub fn new(log_id: LogId, log_entry_ids: Vec<LogEntryId>, server_id: ServerId) -> Self {
        Self {
            log_id,
            log_entry_ids,
            server_id,
        }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.log_entry_ids
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogEntriesRecoveryMessageInput {
    server_id: ServerId,
    log_entry_ids: Vec<LogEntryId>,
}

impl From<ServerLogEntriesRecoveryMessageInput> for (ServerId, Vec<LogEntryId>) {
    fn from(input: ServerLogEntriesRecoveryMessageInput) -> Self {
        (input.server_id, input.log_entry_ids)
    }
}

impl ServerLogEntriesRecoveryMessageInput {
    pub fn new(server_id: ServerId, log_entry_ids: Vec<LogEntryId>) -> Self {
        Self {
            server_id,
            log_entry_ids,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.log_entry_ids
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogEntriesGetMessageInput {
    log_id: LogId,
    log_entry_id: LogEntryId,
}

impl From<ServerLogEntriesGetMessageInput> for (LogId, LogEntryId) {
    fn from(input: ServerLogEntriesGetMessageInput) -> Self {
        (input.log_id, input.log_entry_id)
    }
}

impl ServerLogEntriesGetMessageInput {
    pub fn new(log_id: LogId, log_entry_id: LogEntryId) -> Self {
        Self {
            log_id,
            log_entry_id,
        }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn log_entry_id(&self) -> LogEntryId {
        self.log_entry_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleGetMessageInput {
    module_id: ModuleId,
}

impl From<ServerModuleGetMessageInput> for (ModuleId,) {
    fn from(input: ServerModuleGetMessageInput) -> Self {
        (input.module_id,)
    }
}

impl ServerModuleGetMessageInput {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleLoadMessageInput {
    module: Arc<dyn ServerModule>,
}

impl From<ServerModuleLoadMessageInput> for (Arc<dyn ServerModule>,) {
    fn from(input: ServerModuleLoadMessageInput) -> Self {
        (input.module,)
    }
}

impl ServerModuleLoadMessageInput {
    pub fn new(module: Arc<dyn ServerModule>) -> Self {
        Self { module }
    }

    pub fn module(&self) -> &Arc<dyn ServerModule> {
        &self.module
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleUnloadMessageInput {
    module_id: ModuleId,
}

impl From<ServerModuleUnloadMessageInput> for (ModuleId,) {
    fn from(input: ServerModuleUnloadMessageInput) -> Self {
        (input.module_id,)
    }
}

impl ServerModuleUnloadMessageInput {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerClientRegistrationMessageInput {
    general_socket_sender: MessageSocketSender<GeneralMessage>,
    general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl From<ServerClientRegistrationMessageInput>
    for (
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
    )
{
    fn from(input: ServerClientRegistrationMessageInput) -> Self {
        (input.general_socket_sender, input.general_socket_receiver)
    }
}

impl ServerClientRegistrationMessageInput {
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
pub struct ServerClientResignationMessageInput {
    server_client_id: ServerClientId,
}

impl From<ServerClientResignationMessageInput> for (ServerClientId,) {
    fn from(input: ServerClientResignationMessageInput) -> Self {
        (input.server_client_id,)
    }
}

impl ServerClientResignationMessageInput {
    pub fn new(server_client_id: ServerClientId) -> Self {
        Self { server_client_id }
    }

    pub fn server_client_id(&self) -> ServerClientId {
        self.server_client_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogAppendRequestMessageInput {
    server_id: ServerId,
    log_id: LogId,
    log_entries: Vec<LogEntry>,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_current_term: LogTerm,
}

impl From<ServerLogAppendRequestMessageInput>
    for (ServerId, LogId, Vec<LogEntry>, Option<LogEntryId>, LogTerm)
{
    fn from(input: ServerLogAppendRequestMessageInput) -> Self {
        (
            input.server_id,
            input.log_id,
            input.log_entries,
            input.last_committed_log_entry_id,
            input.log_current_term,
        )
    }
}

impl ServerLogAppendRequestMessageInput {
    pub fn new(
        server_id: ServerId,
        log_id: LogId,
        log_entries: Vec<LogEntry>,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_current_term: LogTerm,
    ) -> Self {
        Self {
            server_id,
            log_id,
            log_entries,
            last_committed_log_entry_id,
            log_current_term,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn log_entries(&self) -> &Vec<LogEntry> {
        &self.log_entries
    }

    pub fn last_committed_log_entry_id(&self) -> &Option<LogEntryId> {
        &self.last_committed_log_entry_id
    }

    pub fn log_current_term(&self) -> LogTerm {
        self.log_current_term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogAppendInitiateMessageInput {
    log_id: LogId,
}

impl From<ServerLogAppendInitiateMessageInput> for (LogId,) {
    fn from(input: ServerLogAppendInitiateMessageInput) -> Self {
        (input.log_id,)
    }
}

impl ServerLogAppendInitiateMessageInput {
    pub fn new(log_id: LogId) -> Self {
        Self { log_id }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerSocketAddressGetMessageInput {
    server_id: ServerId,
}

impl From<ServerServerSocketAddressGetMessageInput> for (ServerId,) {
    fn from(input: ServerServerSocketAddressGetMessageInput) -> Self {
        (input.server_id,)
    }
}

impl ServerServerSocketAddressGetMessageInput {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerIdGetMessageInput {}

impl From<ServerServerIdGetMessageInput> for () {
    fn from(_input: ServerServerIdGetMessageInput) -> Self {}
}

impl ServerServerIdGetMessageInput {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLeaderServerIdGetMessageInput {}

impl From<ServerLeaderServerIdGetMessageInput> for () {
    fn from(_input: ServerLeaderServerIdGetMessageInput) -> Self {}
}

impl ServerLeaderServerIdGetMessageInput {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerNodeMessageSenderGetMessageInput {
    server_id: ServerId,
}

impl From<ServerServerNodeMessageSenderGetMessageInput> for (ServerId,) {
    fn from(input: ServerServerNodeMessageSenderGetMessageInput) -> Self {
        (input.server_id,)
    }
}

impl ServerServerNodeMessageSenderGetMessageInput {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerNodeMessageSenderGetAllMessageInput {
}

impl From<ServerServerNodeMessageSenderGetAllMessageInput> for () {
    fn from(_input: ServerServerNodeMessageSenderGetAllMessageInput) -> Self {
        ()
    }
}

impl ServerServerNodeMessageSenderGetAllMessageInput {
    pub fn new() -> Self {
        Self { }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerCommitMessageInput {
    log_id: LogId,
    data: Bytes,
}

impl From<ServerCommitMessageInput> for (LogId, Bytes) {
    fn from(input: ServerCommitMessageInput) -> Self {
        (input.log_id, input.data)
    }
}

impl ServerCommitMessageInput {
    pub fn new(log_id: LogId, data: Bytes) -> Self {
        Self { log_id, data }
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn data(&self) -> &Bytes {
        &self.data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerCommitCommitMessageInput {
    output_sender: SynchronizableMessageOutputSender,
}

impl From<ServerCommitCommitMessageInput> for (SynchronizableMessageOutputSender,) {
    fn from(input: ServerCommitCommitMessageInput) -> Self {
        (input.output_sender,)
    }
}

impl ServerCommitCommitMessageInput {
    pub fn new(output_sender: SynchronizableMessageOutputSender) -> Self {
        Self { output_sender }
    }

    pub fn output_sender(&self) -> &SynchronizableMessageOutputSender {
        &self.output_sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerSocketAddressesGetMessageInput {}

impl From<ServerServerSocketAddressesGetMessageInput> for () {
    fn from(_input: ServerServerSocketAddressesGetMessageInput) -> Self {
        ()
    }
}

impl ServerServerSocketAddressesGetMessageInput {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogCreateMessageInput {
    log_issuer: Option<LogIssuer>,
}

impl From<ServerLogCreateMessageInput> for (Option<LogIssuer>,) {
    fn from(input: ServerLogCreateMessageInput) -> Self {
        (input.log_issuer,)
    }
}

impl ServerLogCreateMessageInput {
    pub fn new(log_issuer: Option<LogIssuer>) -> Self {
        Self { log_issuer }
    }

    pub fn log_issuer(&self) -> &Option<LogIssuer> {
        &self.log_issuer
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerCommitLogCreateMessageInput {
    output_sender: SynchronousMessageOutputSender<ServerLogCreateMessageOutput>,
    log_id: LogId,
}

impl From<ServerCommitLogCreateMessageInput>
    for (
        SynchronousMessageOutputSender<ServerLogCreateMessageOutput>,
        LogId,
    )
{
    fn from(input: ServerCommitLogCreateMessageInput) -> Self {
        (input.output_sender, input.log_id)
    }
}

impl ServerCommitLogCreateMessageInput {
    pub fn new(
        output_sender: SynchronousMessageOutputSender<ServerLogCreateMessageOutput>,
        log_id: LogId,
    ) -> Self {
        Self {
            output_sender,
            log_id,
        }
    }

    pub fn output_sender(&self) -> &SynchronousMessageOutputSender<ServerLogCreateMessageOutput> {
        &self.output_sender
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }
}
