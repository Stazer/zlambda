use crate::common::module::ModuleId;
use crate::common::notification::NotificationBodyItemQueueReceiver;
use crate::server::client::ServerClientId;
use crate::server::{LogId, LogEntryId, Server, ServerId};
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleStartupEventInput {
    server: Arc<Server>,
}

impl From<ServerModuleStartupEventInput> for (Arc<Server>,) {
    fn from(input: ServerModuleStartupEventInput) -> Self {
        (input.server,)
    }
}

impl ServerModuleStartupEventInput {
    pub fn new(server: Arc<Server>) -> Self {
        Self { server }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleShutdownEventInput {
    server: Arc<Server>,
}

impl From<ServerModuleShutdownEventInput> for (Arc<Server>,) {
    fn from(input: ServerModuleShutdownEventInput) -> Self {
        (input.server,)
    }
}

impl ServerModuleShutdownEventInput {
    pub fn new(server: Arc<Server>) -> Self {
        Self { server }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleLoadEventInput {
    module_id: ModuleId,
    server: Arc<Server>,
}

impl From<ServerModuleLoadEventInput> for (ModuleId, Arc<Server>) {
    fn from(input: ServerModuleLoadEventInput) -> Self {
        (input.module_id, input.server)
    }
}

impl ServerModuleLoadEventInput {
    pub fn new(module_id: ModuleId, server: Arc<Server>) -> Self {
        Self { module_id, server }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleUnloadEventInput {
    server: Arc<Server>,
}

impl From<ServerModuleUnloadEventInput> for (Arc<Server>,) {
    fn from(input: ServerModuleUnloadEventInput) -> Self {
        (input.server,)
    }
}

impl ServerModuleUnloadEventInput {
    pub fn new(server: Arc<Server>) -> Self {
        Self { server }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Arc<Server> {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleNotificationEventInputServerSource {
    server_id: ServerId,
}

impl From<ServerModuleNotificationEventInputServerSource> for (ServerId,) {
    fn from(source: ServerModuleNotificationEventInputServerSource) -> Self {
        (source.server_id,)
    }
}

impl ServerModuleNotificationEventInputServerSource {
    pub fn new(server_id: ServerId) -> Self {
        Self { server_id }
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
    source: ServerModuleNotificationEventInputSource,
    notification_body_item_queue_receiver: NotificationBodyItemQueueReceiver,
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
        source: ServerModuleNotificationEventInputSource,
        notification_body_item_queue_receiver: NotificationBodyItemQueueReceiver,
    ) -> Self {
        Self {
            server,
            source,
            notification_body_item_queue_receiver,
        }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
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

impl ServerModuleCommitEventInput {
    pub fn new(server: Arc<Server>, log_id: LogId, log_entry_id: LogEntryId) -> Self {
        Self {
            server,
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

    pub fn log_id(&self) -> LogId {
        self.log_id
    }

    pub fn log_entry_id(&self) -> LogEntryId {
        self.log_entry_id
    }
}
