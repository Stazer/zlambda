use crate::common::message::MessageQueueSender;
use crate::common::module::{LoadModuleError, ModuleId, UnloadModuleError};
use crate::server::node::ServerNodeMessage;
use crate::server::{LogEntryId, LogTerm, ServerId, ServerModule};
use std::net::SocketAddr;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRegistrationMessageNotALeaderOutput {
    leader_server_socket_address: SocketAddr,
}

impl From<ServerRegistrationMessageNotALeaderOutput> for (SocketAddr,) {
    fn from(output: ServerRegistrationMessageNotALeaderOutput) -> Self {
        (output.leader_server_socket_address,)
    }
}

impl ServerRegistrationMessageNotALeaderOutput {
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

#[derive(Debug)]
pub struct ServerRegistrationMessageSuccessOutput {
    server_id: ServerId,
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    last_committed_log_entry_id: Option<LogEntryId>,
    current_log_term: LogTerm,
    node_sender: MessageQueueSender<ServerNodeMessage>,
}

impl From<ServerRegistrationMessageSuccessOutput>
    for (
        ServerId,
        ServerId,
        Vec<Option<SocketAddr>>,
        Option<LogEntryId>,
        LogTerm,
        MessageQueueSender<ServerNodeMessage>,
    )
{
    fn from(output: ServerRegistrationMessageSuccessOutput) -> Self {
        (
            output.server_id,
            output.leader_server_id,
            output.server_socket_addresses,
            output.last_committed_log_entry_id,
            output.current_log_term,
            output.node_sender,
        )
    }
}

impl ServerRegistrationMessageSuccessOutput {
    pub fn new(
        server_id: ServerId,
        leader_server_id: ServerId,
        server_socket_addresses: Vec<Option<SocketAddr>>,
        last_committed_log_entry_id: Option<LogEntryId>,
        current_log_term: LogTerm,
        node_sender: MessageQueueSender<ServerNodeMessage>,
    ) -> Self {
        Self {
            server_id,
            leader_server_id,
            server_socket_addresses,
            last_committed_log_entry_id,
            current_log_term,
            node_sender,
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

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn curremt_log_term(&self) -> LogTerm {
        self.current_log_term
    }

    pub fn node_sender(&self) -> &MessageQueueSender<ServerNodeMessage> {
        &self.node_sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerRegistrationMessageOutput {
    NotALeader(ServerRegistrationMessageNotALeaderOutput),
    Success(ServerRegistrationMessageSuccessOutput),
}

impl From<ServerRegistrationMessageNotALeaderOutput> for ServerRegistrationMessageOutput {
    fn from(output: ServerRegistrationMessageNotALeaderOutput) -> Self {
        Self::NotALeader(output)
    }
}

impl From<ServerRegistrationMessageSuccessOutput> for ServerRegistrationMessageOutput {
    fn from(output: ServerRegistrationMessageSuccessOutput) -> Self {
        Self::Success(output)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRecoveryMessageNotALeaderOutput {
    leader_server_socket_address: SocketAddr,
}

impl From<ServerRecoveryMessageNotALeaderOutput> for (SocketAddr,) {
    fn from(output: ServerRecoveryMessageNotALeaderOutput) -> Self {
        (output.leader_server_socket_address,)
    }
}

impl ServerRecoveryMessageNotALeaderOutput {
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

#[derive(Debug)]
pub struct ServerRecoveryMessageSuccessOutput {
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    last_committed_log_entry_id: Option<LogEntryId>,
    current_log_term: LogTerm,
    node_queue_sender: MessageQueueSender<ServerNodeMessage>,
}

impl From<ServerRecoveryMessageSuccessOutput>
    for (
        ServerId,
        Vec<Option<SocketAddr>>,
        Option<LogEntryId>,
        LogTerm,
        MessageQueueSender<ServerNodeMessage>,
    )
{
    fn from(output: ServerRecoveryMessageSuccessOutput) -> Self {
        (
            output.leader_server_id,
            output.server_socket_addresses,
            output.last_committed_log_entry_id,
            output.current_log_term,
            output.node_queue_sender,
        )
    }
}

impl ServerRecoveryMessageSuccessOutput {
    pub fn new(
        leader_server_id: ServerId,
        server_socket_addresses: Vec<Option<SocketAddr>>,
        last_committed_log_entry_id: Option<LogEntryId>,
        current_log_term: LogTerm,
        node_queue_sender: MessageQueueSender<ServerNodeMessage>,
    ) -> Self {
        Self {
            leader_server_id,
            server_socket_addresses,
            last_committed_log_entry_id,
            current_log_term,
            node_queue_sender,
        }
    }

    pub fn leader_server_id(&self) -> ServerId {
        self.leader_server_id
    }

    pub fn server_socket_addresses(&self) -> &Vec<Option<SocketAddr>> {
        &self.server_socket_addresses
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn current_log_term(&self) -> LogTerm {
        self.current_log_term
    }

    pub fn node_queue_sender(&self) -> &MessageQueueSender<ServerNodeMessage> {
        &self.node_queue_sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerRecoveryMessageOutput {
    NotALeader(ServerRecoveryMessageNotALeaderOutput),
    IsOnline,
    Unknown,
    Success(ServerRecoveryMessageSuccessOutput),
}

impl From<ServerRecoveryMessageNotALeaderOutput> for ServerRecoveryMessageOutput {
    fn from(output: ServerRecoveryMessageNotALeaderOutput) -> Self {
        Self::NotALeader(output)
    }
}

impl From<ServerRecoveryMessageSuccessOutput> for ServerRecoveryMessageOutput {
    fn from(output: ServerRecoveryMessageSuccessOutput) -> Self {
        Self::Success(output)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLogEntriesReplicationMessageOutput {
    log_entry_ids: Vec<LogEntryId>,
}

impl From<ServerLogEntriesReplicationMessageOutput> for (Vec<LogEntryId>,) {
    fn from(output: ServerLogEntriesReplicationMessageOutput) -> Self {
        (output.log_entry_ids,)
    }
}

impl ServerLogEntriesReplicationMessageOutput {
    pub fn new(log_entry_ids: Vec<LogEntryId>) -> Self {
        Self { log_entry_ids }
    }

    pub fn log_entry_ids(&self) -> &Vec<LogEntryId> {
        &self.log_entry_ids
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleGetMessageOutput {
    module: Option<Arc<dyn ServerModule>>,
}

impl From<ServerModuleGetMessageOutput> for (Option<Arc<dyn ServerModule>>,) {
    fn from(output: ServerModuleGetMessageOutput) -> Self {
        (output.module,)
    }
}

impl ServerModuleGetMessageOutput {
    pub fn new(module: Option<Arc<dyn ServerModule>>) -> Self {
        Self { module }
    }

    pub fn module(&self) -> &Option<Arc<dyn ServerModule>> {
        &self.module
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleLoadMessageOutput {
    result: Result<ModuleId, LoadModuleError>,
}

impl From<ServerModuleLoadMessageOutput> for (Result<ModuleId, LoadModuleError>,) {
    fn from(output: ServerModuleLoadMessageOutput) -> Self {
        (output.result,)
    }
}

impl ServerModuleLoadMessageOutput {
    pub fn new(result: Result<ModuleId, LoadModuleError>) -> Self {
        Self { result }
    }

    pub fn result(&self) -> &Result<ModuleId, LoadModuleError> {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleUnloadMessageOutput {
    result: Result<(), UnloadModuleError>,
}

impl From<ServerModuleUnloadMessageOutput> for (Result<(), UnloadModuleError>,) {
    fn from(output: ServerModuleUnloadMessageOutput) -> Self {
        (output.result,)
    }
}

impl ServerModuleUnloadMessageOutput {
    pub fn new(result: Result<(), UnloadModuleError>) -> Self {
        Self { result }
    }

    pub fn result(&self) -> &Result<(), UnloadModuleError> {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerSocketAddressGetMessageOutput {
    socket_address: Option<SocketAddr>,
}

impl From<ServerServerSocketAddressGetMessageOutput> for (Option<SocketAddr>,) {
    fn from(output: ServerServerSocketAddressGetMessageOutput) -> Self {
        (output.socket_address,)
    }
}

impl ServerServerSocketAddressGetMessageOutput {
    pub fn new(socket_address: Option<SocketAddr>) -> Self {
        Self { socket_address }
    }

    pub fn socket_address(&self) -> Option<SocketAddr> {
        self.socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerIdGetMessageOutput {
    server_id: ServerId,
}

impl From<ServerServerIdGetMessageOutput> for (ServerId,) {
    fn from(output: ServerServerIdGetMessageOutput) -> Self {
        (output.server_id,)
    }
}

impl ServerServerIdGetMessageOutput {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerLeaderServerIdGetMessageOutput {
    leader_server_id: ServerId,
}

impl From<ServerLeaderServerIdGetMessageOutput> for (ServerId,) {
    fn from(output: ServerLeaderServerIdGetMessageOutput) -> Self {
        (output.leader_server_id,)
    }
}

impl ServerLeaderServerIdGetMessageOutput {
    pub fn new(leader_server_id: ServerId) -> Self {
        Self { leader_server_id }
    }

    pub fn leader_server_id(&self) -> ServerId {
        self.leader_server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerNodeMessageSenderGetMessageOutput {
    server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
}

impl From<ServerServerNodeMessageSenderGetMessageOutput>
    for (Option<MessageQueueSender<ServerNodeMessage>>,)
{
    fn from(input: ServerServerNodeMessageSenderGetMessageOutput) -> Self {
        (input.server_node_message_sender,)
    }
}

impl ServerServerNodeMessageSenderGetMessageOutput {
    pub fn new(server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>) -> Self {
        Self {
            server_node_message_sender,
        }
    }

    pub fn server_node_message_sender(&self) -> &Option<MessageQueueSender<ServerNodeMessage>> {
        &self.server_node_message_sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerServerSocketAddressesGetMessageOutput {
    server_socket_addresses: Vec<Option<SocketAddr>>,
}

impl From<ServerServerSocketAddressesGetMessageOutput> for (Vec<Option<SocketAddr>>,) {
    fn from(output: ServerServerSocketAddressesGetMessageOutput) -> Self {
        (output.server_socket_addresses,)
    }
}

impl ServerServerSocketAddressesGetMessageOutput {
    pub fn new(
        server_socket_addresses: Vec<Option<SocketAddr>>,
    ) -> Self {
        Self {
            server_socket_addresses,
        }
    }

    pub fn server_socket_addresses(&self) -> &Vec<Option<SocketAddr>> {
        &self.server_socket_addresses
    }
}
