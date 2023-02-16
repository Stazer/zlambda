use crate::common::message::MessageQueueReceiver;
use crate::common::module::ModuleId;
use crate::common::utility::Bytes;
use crate::server::client::ServerClientId;
use crate::server::{LogEntryId, ServerHandle, ServerId};
use async_stream::stream;
use futures::Stream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleStartupEventInput {
    server: ServerHandle,
}

impl From<ServerModuleStartupEventInput> for (ServerHandle,) {
    fn from(input: ServerModuleStartupEventInput) -> Self {
        (input.server,)
    }
}

impl ServerModuleStartupEventInput {
    pub fn new(server: ServerHandle) -> Self {
        Self { server }
    }

    pub fn server(&self) -> &ServerHandle {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut ServerHandle {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleShutdownEventInput {
    server: ServerHandle,
}

impl From<ServerModuleShutdownEventInput> for (ServerHandle,) {
    fn from(input: ServerModuleShutdownEventInput) -> Self {
        (input.server,)
    }
}

impl ServerModuleShutdownEventInput {
    pub fn new(server: ServerHandle) -> Self {
        Self { server }
    }

    pub fn server(&self) -> &ServerHandle {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut ServerHandle {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleLoadEventInput {
    module_id: ModuleId,
    server: ServerHandle,
}

impl From<ServerModuleLoadEventInput> for (ModuleId, ServerHandle) {
    fn from(input: ServerModuleLoadEventInput) -> Self {
        (input.module_id, input.server)
    }
}

impl ServerModuleLoadEventInput {
    pub fn new(module_id: ModuleId, server: ServerHandle) -> Self {
        Self { module_id, server }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn server(&self) -> &ServerHandle {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut ServerHandle {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleUnloadEventInput {
    server: ServerHandle,
}

impl From<ServerModuleUnloadEventInput> for (ServerHandle,) {
    fn from(input: ServerModuleUnloadEventInput) -> Self {
        (input.server,)
    }
}

impl ServerModuleUnloadEventInput {
    pub fn new(server: ServerHandle) -> Self {
        Self { server }
    }

    pub fn server(&self) -> &ServerHandle {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut ServerHandle {
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
pub struct ServerModuleNotificationEventBody {
    receiver: MessageQueueReceiver<Bytes>,
}

impl ServerModuleNotificationEventBody {
    pub fn new(receiver: MessageQueueReceiver<Bytes>) -> Self {
        Self { receiver }
    }

    pub fn stream(&mut self) -> impl Stream<Item = Bytes> + '_ {
        stream!(while let Some(bytes) = self.receiver.receive().await {
            yield bytes
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleNotificationEventInput {
    server: ServerHandle,
    source: ServerModuleNotificationEventInputSource,
    body: ServerModuleNotificationEventBody,
}

impl From<ServerModuleNotificationEventInput>
    for (
        ServerHandle,
        ServerModuleNotificationEventInputSource,
        ServerModuleNotificationEventBody,
    )
{
    fn from(input: ServerModuleNotificationEventInput) -> Self {
        (input.server, input.source, input.body)
    }
}

impl ServerModuleNotificationEventInput {
    pub fn new(
        server: ServerHandle,
        source: ServerModuleNotificationEventInputSource,
        body: ServerModuleNotificationEventBody,
    ) -> Self {
        Self {
            server,
            source,
            body,
        }
    }

    pub fn server(&self) -> &ServerHandle {
        &self.server
    }

    pub fn source(&self) -> &ServerModuleNotificationEventInputSource {
        &self.source
    }

    pub fn body(&self) -> &ServerModuleNotificationEventBody {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut ServerModuleNotificationEventBody {
        &mut self.body
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModuleCommitEventInput {
    server: ServerHandle,
    log_entry_id: LogEntryId,
}

impl From<ServerModuleCommitEventInput> for (ServerHandle, LogEntryId) {
    fn from(input: ServerModuleCommitEventInput) -> Self {
        (input.server, input.log_entry_id)
    }
}

impl ServerModuleCommitEventInput {
    pub fn new(server: ServerHandle, log_entry_id: LogEntryId) -> Self {
        Self {
            server,
            log_entry_id,
        }
    }

    pub fn server(&self) -> &ServerHandle {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut ServerHandle {
        &mut self.server
    }

    pub fn log_entry_id(&self) -> LogEntryId {
        self.log_entry_id
    }
}
