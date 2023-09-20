use crate::common::module::ModuleId;
use crate::common::notification::NotificationBodyItemQueueReceiver;
use crate::server::client::ServerClientId;
use crate::server::{LogEntryId, LogId, Server, ServerId};
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleStartupEventInput {
    server: Arc<Server>,
    module_id: ModuleId,
}

impl From<ServerModuleStartupEventInput> for (Arc<Server>, ModuleId) {
    fn from(input: ServerModuleStartupEventInput) -> Self {
        (input.server, input.module_id)
    }
}

impl ServerModuleStartupEventInput {
    pub fn new(server: Arc<Server>, module_id: ModuleId) -> Self {
        Self { server, module_id }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleShutdownEventInput {
    server: Arc<Server>,
    module_id: ModuleId,
}

impl From<ServerModuleShutdownEventInput> for (Arc<Server>, ModuleId) {
    fn from(input: ServerModuleShutdownEventInput) -> Self {
        (input.server, input.module_id)
    }
}

impl ServerModuleShutdownEventInput {
    pub fn new(server: Arc<Server>, module_id: ModuleId) -> Self {
        Self { server, module_id }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleLoadEventInput {
    server: Arc<Server>,
    module_id: ModuleId,
}

impl From<ServerModuleLoadEventInput> for (Arc<Server>, ModuleId) {
    fn from(input: ServerModuleLoadEventInput) -> Self {
        (input.server, input.module_id)
    }
}

impl ServerModuleLoadEventInput {
    pub fn new(server: Arc<Server>, module_id: ModuleId) -> Self {
        Self { server, module_id }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleUnloadEventInput {
    server: Arc<Server>,
    module_id: ModuleId,
}

impl From<ServerModuleUnloadEventInput> for (Arc<Server>,) {
    fn from(input: ServerModuleUnloadEventInput) -> Self {
        (input.server,)
    }
}

impl ServerModuleUnloadEventInput {
    pub fn new(server: Arc<Server>, module_id: ModuleId) -> Self {
        Self { server, module_id }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleNotificationEventInputServerSourceOrigin {
    server_id: ServerId,
    server_client_id: ServerClientId,
}

impl ServerModuleNotificationEventInputServerSourceOrigin {
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

#[derive(Clone, Debug)]
pub struct ServerModuleNotificationEventInputServerSource {
    origin: Option<ServerModuleNotificationEventInputServerSourceOrigin>,
    server_id: ServerId,
}

impl From<ServerModuleNotificationEventInputServerSource> for (ServerId,) {
    fn from(source: ServerModuleNotificationEventInputServerSource) -> Self {
        (source.server_id,)
    }
}

impl From<ServerModuleNotificationEventInputServerSource>
    for (
        Option<ServerModuleNotificationEventInputServerSourceOrigin>,
        ServerId,
    )
{
    fn from(source: ServerModuleNotificationEventInputServerSource) -> Self {
        (source.origin, source.server_id)
    }
}

impl ServerModuleNotificationEventInputServerSource {
    pub fn new(
        origin: Option<ServerModuleNotificationEventInputServerSourceOrigin>,
        server_id: ServerId,
    ) -> Self {
        Self { origin, server_id }
    }

    pub fn origin(&self) -> &Option<ServerModuleNotificationEventInputServerSourceOrigin> {
        &self.origin
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleNotificationEventInputClientSource {
    server_client_id: ServerClientId,
}

impl From<ServerModuleNotificationEventInputClientSource> for (ServerClientId,) {
    fn from(source: ServerModuleNotificationEventInputClientSource) -> Self {
        (source.server_client_id,)
    }
}

impl ServerModuleNotificationEventInputClientSource {
    pub fn new(server_client_id: ServerClientId) -> Self {
        Self { server_client_id }
    }

    pub fn server_client_id(&self) -> ServerClientId {
        self.server_client_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum ServerModuleNotificationEventInputSource {
    Server(ServerModuleNotificationEventInputServerSource),
    Client(ServerModuleNotificationEventInputClientSource),
}

impl From<ServerModuleNotificationEventInputServerSource>
    for ServerModuleNotificationEventInputSource
{
    fn from(source: ServerModuleNotificationEventInputServerSource) -> Self {
        Self::Server(source)
    }
}

impl From<ServerModuleNotificationEventInputClientSource>
    for ServerModuleNotificationEventInputSource
{
    fn from(source: ServerModuleNotificationEventInputClientSource) -> Self {
        Self::Client(source)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleNotificationEventInput {
    server: Arc<Server>,
    module_id: ModuleId,
    source: ServerModuleNotificationEventInputSource,
    notification_body_item_queue_receiver: NotificationBodyItemQueueReceiver,
}

impl From<ServerModuleNotificationEventInput>
    for (
        Arc<Server>,
        ModuleId,
        ServerModuleNotificationEventInputSource,
        NotificationBodyItemQueueReceiver,
    )
{
    fn from(input: ServerModuleNotificationEventInput) -> Self {
        (
            input.server,
            input.module_id,
            input.source,
            input.notification_body_item_queue_receiver,
        )
    }
}

impl From<ServerModuleNotificationEventInput>
    for (
        Arc<Server>,
        ServerModuleNotificationEventInputSource,
        NotificationBodyItemQueueReceiver,
    )
{
    fn from(input: ServerModuleNotificationEventInput) -> Self {
        (
            input.server,
            input.source,
            input.notification_body_item_queue_receiver,
        )
    }
}

impl ServerModuleNotificationEventInput {
    pub fn new(
        server: Arc<Server>,
        module_id: ModuleId,
        source: ServerModuleNotificationEventInputSource,
        notification_body_item_queue_receiver: NotificationBodyItemQueueReceiver,
    ) -> Self {
        Self {
            server,
            module_id,
            source,
            notification_body_item_queue_receiver,
        }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn source(&self) -> &ServerModuleNotificationEventInputSource {
        &self.source
    }

    pub fn notification_body_item_queue_receiver(&self) -> &NotificationBodyItemQueueReceiver {
        &self.notification_body_item_queue_receiver
    }

    pub fn notification_body_item_queue_receiver_mut(
        &mut self,
    ) -> &mut NotificationBodyItemQueueReceiver {
        &mut self.notification_body_item_queue_receiver
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleCommitEventInput {
    server: Arc<Server>,
    module_id: ModuleId,
    log_id: LogId,
    log_entry_id: LogEntryId,
}

impl From<ServerModuleCommitEventInput> for (Arc<Server>, LogEntryId) {
    fn from(input: ServerModuleCommitEventInput) -> Self {
        (input.server, input.log_entry_id)
    }
}

impl From<ServerModuleCommitEventInput> for (Arc<Server>, LogId, LogEntryId) {
    fn from(input: ServerModuleCommitEventInput) -> Self {
        (input.server, input.log_id, input.log_entry_id)
    }
}

impl From<ServerModuleCommitEventInput> for (Arc<Server>, ModuleId, LogId, LogEntryId) {
    fn from(input: ServerModuleCommitEventInput) -> Self {
        (
            input.server,
            input.module_id,
            input.log_id,
            input.log_entry_id,
        )
    }
}

impl ServerModuleCommitEventInput {
    pub fn new(
        server: Arc<Server>,
        module_id: ModuleId,
        log_id: LogId,
        log_entry_id: LogEntryId,
    ) -> Self {
        Self {
            server,
            module_id,
            log_id,
            log_entry_id,
        }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn log_entry_id(&self) -> LogEntryId {
        self.log_entry_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModulePromotionEventInput {
    concerning_log_id: LogId,
    promoted_server_id: ServerId,
    server: Arc<Server>,
}

impl From<ServerModulePromotionEventInput> for (LogId, ServerId, Arc<Server>) {
    fn from(input: ServerModulePromotionEventInput) -> Self {
        (
            input.concerning_log_id,
            input.promoted_server_id,
            input.server,
        )
    }
}

impl ServerModulePromotionEventInput {
    pub fn new(
        concerning_log_id: LogId,
        promoted_server_id: ServerId,
        server: Arc<Server>,
    ) -> Self {
        Self {
            concerning_log_id,
            promoted_server_id,
            server,
        }
    }

    pub fn concerning_log_id(&self) -> LogId {
        self.concerning_log_id
    }

    pub fn promoted_server_id(&self) -> ServerId {
        self.promoted_server_id
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleServerConnectEventInput {
    server: Arc<Server>,
    server_id: ServerId,
}

impl From<ServerModuleServerConnectEventInput> for (Arc<Server>, ServerId) {
    fn from(input: ServerModuleServerConnectEventInput) -> Self {
        (input.server, input.server_id)
    }
}

impl ServerModuleServerConnectEventInput {
    pub fn new(server: Arc<Server>, server_id: ServerId) -> Self {
        Self { server, server_id }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleServerDisconnectEventInput {
    server: Arc<Server>,
    server_id: ServerId,
}

impl From<ServerModuleServerDisconnectEventInput> for (Arc<Server>, ServerId) {
    fn from(input: ServerModuleServerDisconnectEventInput) -> Self {
        (input.server, input.server_id)
    }
}

impl ServerModuleServerDisconnectEventInput {
    pub fn new(server: Arc<Server>, server_id: ServerId) -> Self {
        Self { server, server_id }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }
}
