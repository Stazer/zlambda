mod client;
mod connection;
mod error;
mod id;
mod log;
mod message;
mod module;
mod node;
mod task;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use client::*;
pub use connection::*;
pub use error::*;
pub use id::*;
pub use log::*;
pub use message::*;
pub use module::*;
pub use node::*;
use task::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::message::MessageQueueSender;
use crate::common::module::{LoadModuleError, ModuleId, UnloadModuleError};
use crate::common::net::ToSocketAddrs;
use crate::common::notification::notification_body_item_queue;
use crate::common::runtime::spawn;
use crate::common::utility::Bytes;
use crate::general::{
    GeneralClientRedirectMessage, GeneralClientRedirectMessageInput, GeneralMessage,
    GeneralNotificationMessage, GeneralNotificationMessageInput,
    GeneralNotificationMessageInputImmediateType, GeneralNotificationMessageInputType,
};
use futures::{Stream, StreamExt};
use postcard::to_allocvec;
use std::fmt::Debug;
use std::future::pending;
use std::iter::once;
use std::net::SocketAddr;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IntoArcServerModule {
    fn into_arc_server_module(self) -> Arc<dyn ServerModule>;
}

impl IntoArcServerModule for Arc<dyn ServerModule> {
    fn into_arc_server_module(self) -> Arc<dyn ServerModule> {
        self
    }
}

impl IntoArcServerModule for Box<dyn ServerModule> {
    fn into_arc_server_module(self) -> Arc<dyn ServerModule> {
        Arc::from(self)
    }
}

impl<T> IntoArcServerModule for T
where
    T: ServerModule + 'static,
{
    fn into_arc_server_module(self) -> Arc<dyn ServerModule> {
        Arc::from(self)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct ServerBuilder {
    modules: Vec<Arc<dyn ServerModule>>,
}

impl ServerBuilder {
    pub fn add_module<T>(mut self, module: T) -> Self
    where
        T: IntoArcServerModule,
    {
        self.modules.push(module.into_arc_server_module());

        self
    }

    pub async fn build<S, T>(
        self,
        listener_address: S,
        follower_data: Option<(T, Option<ServerId>)>,
    ) -> Result<Arc<Server>, NewServerError>
    where
        S: ToSocketAddrs + Debug,
        T: ToSocketAddrs + Debug,
    {
        let task =
            ServerTask::new(listener_address, follower_data, self.modules.into_iter()).await?;

        let server = task.server().clone();

        task.spawn();

        Ok(server)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Server {
    server_message_sender: MessageQueueSender<ServerMessage>,
}

impl From<MessageQueueSender<ServerMessage>> for Server {
    fn from(server_message_sender: MessageQueueSender<ServerMessage>) -> Self {
        Self {
            server_message_sender,
        }
    }
}

impl Server {
    pub(crate) fn new(server_message_sender: MessageQueueSender<ServerMessage>) -> Arc<Self> {
        Arc::new(Self {
            server_message_sender,
        })
    }

    pub(crate) fn server_message_sender(&self) -> &MessageQueueSender<ServerMessage> {
        &self.server_message_sender
    }

    pub async fn server_id(&self) -> ServerId {
        let output = self
            .server_message_sender
            .do_send_synchronous(ServerServerIdGetMessageInput::new())
            .await;

        let (server_id,) = output.into();

        server_id
    }

    pub async fn leader_server_id(&self) -> ServerId {
        let output = self
            .server_message_sender
            .do_send_synchronous(ServerLeaderServerIdGetMessageInput::new())
            .await;

        let (leader_server_id,) = output.into();

        leader_server_id
    }

    pub fn modules(&self) -> ServerModules<'_> {
        ServerModules::new(self)
    }

    pub fn servers(&self) -> ServerServers<'_> {
        ServerServers::new(self)
    }

    pub fn logs(&self) -> ServerLogs<'_> {
        ServerLogs::new(self)
    }

    pub fn local_clients(&self) -> ServerLocalClients<'_> {
        ServerLocalClients::new(self)
    }

    pub async fn notify<T>(
        &self,
        module_id: ModuleId,
        mut body: T,
        origin: Option<ServerNotificationOrigin>,
    ) where
        T: Stream<Item = Bytes> + Unpin + Send + 'static,
    {
        let output = self
            .server_message_sender
            .do_send_synchronous(ServerModuleGetMessageInput::new(module_id))
            .await;

        let (module,) = match output.into() {
            (Some(module),) => (module,),
            (None,) => return,
        };

        let (sender, receiver) = notification_body_item_queue();
        let server_source = ServerModuleNotificationEventInputServerSource::new(
            origin.map(|o| {
                ServerModuleNotificationEventInputServerSourceOrigin::new(
                    o.server_id(),
                    o.server_client_id(),
                )
            }),
            self.server_id().await,
        );

        spawn(async move {
            while let Some(body) = body.next().await {
                sender.do_send(body).await;
            }
        });

        module
            .on_notification(ServerModuleNotificationEventInput::new(
                Server::new(self.server_message_sender.clone()),
                module_id,
                server_source.into(),
                receiver,
            ))
            .await;
    }

    pub async fn wait(&self) {
        pending::<()>().await;
        todo!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerModules<'a> {
    server: &'a Server,
}

impl<'a> ServerModules<'a> {
    pub(crate) fn new(server: &'a Server) -> Self {
        Self { server }
    }

    pub async fn load<T>(&self, module: T) -> Result<ModuleId, LoadModuleError>
    where
        T: IntoArcServerModule,
    {
        let output = self
            .server
            .server_message_sender()
            .do_send_synchronous(ServerModuleLoadMessageInput::new(
                module.into_arc_server_module(),
            ))
            .await;

        let (result,) = output.into();

        result
    }

    pub async fn unload(&self, module_id: ModuleId) -> Result<(), UnloadModuleError> {
        let output = self
            .server
            .server_message_sender()
            .do_send_synchronous(ServerModuleUnloadMessageInput::new(module_id))
            .await;

        let (result,) = output.into();

        result
    }

    pub async fn get(&self, module_id: ModuleId) -> Option<Arc<dyn ServerModule>> {
        let output = self
            .server
            .server_message_sender()
            .do_send_synchronous(ServerModuleGetMessageInput::new(module_id))
            .await;

        let (result,) = output.into();

        result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerServers<'a> {
    server: &'a Server,
}

impl<'a> ServerServers<'a> {
    pub(crate) fn new(server: &'a Server) -> Self {
        Self { server }
    }

    pub(crate) fn server(&self) -> &Server {
        self.server
    }

    pub async fn socket_addresses(&self) -> Vec<Option<SocketAddr>> {
        let output = self
            .server()
            .server_message_sender()
            .do_send_synchronous(ServerServerSocketAddressesGetMessageInput::new())
            .await;
        let (socket_addresses,) = output.into();

        socket_addresses
    }

    pub async fn get(&self, server_id: ServerId) -> Option<ServerServersServer<'_>> {
        if self.server().server_id().await == server_id {
            Some(ServerServersServer::new(server_id, self.server(), None))
        } else {
            let output = self
                .server
                .server_message_sender()
                .do_send_synchronous(ServerServerNodeMessageSenderGetMessageInput::new(server_id))
                .await;

            let (server_node_message_sender,) = output.into();

            server_node_message_sender.map(|server_node_message_sender| {
                ServerServersServer::new(server_id, self.server(), Some(server_node_message_sender))
            })
        }
    }

    pub async fn iter(&self) -> impl Iterator<Item = ServerServersServer<'_>> {
        let output = self
            .server
            .server_message_sender()
            .do_send_synchronous(ServerServerNodeMessageSenderGetAllMessageInput::new())
            .await;

        let (server_node_message_senders,) = output.into();

        once(ServerServersServer::new(
            self.server().server_id().await,
            self.server(),
            None,
        ))
        .chain(server_node_message_senders.into_iter().map(
            |(server_id, server_node_message_sender)| {
                ServerServersServer::new(server_id, self.server(), Some(server_node_message_sender))
            },
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerNotificationOrigin {
    server_id: ServerId,
    server_client_id: ServerClientId,
}

impl ServerNotificationOrigin {
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
pub struct ServerNotificationRedirection {
    server_id: ServerId,
    server_client_id: Option<ServerClientId>,
}

impl ServerNotificationRedirection {
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

#[derive(Clone, Debug)]
pub struct ServerServersServer<'a> {
    id: ServerId,
    server: &'a Server,
    server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
}

impl<'a> ServerServersServer<'a> {
    pub(crate) fn new(
        id: ServerId,
        server: &'a Server,
        server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
    ) -> Self {
        Self {
            id,
            server,
            server_node_message_sender,
        }
    }

    pub fn id(&self) -> ServerId {
        self.id
    }

    pub fn clients(&self) -> ServerServersServerClients<'_> {
        ServerServersServerClients::new(
            self.id,
            self.server,
            self.server_node_message_sender.clone(),
        )
    }

    pub async fn send(&self, bytes: Bytes) {
        self.server_node_message_sender
            .as_ref()
            .expect("node")
            .do_send_asynchronous(ServerNodeSendMessageInput::new(bytes))
            .await;
    }

    pub async fn notify<T>(
        &self,
        module_id: ModuleId,
        mut body: T,
        origin: Option<ServerNotificationOrigin>,
    ) where
        T: Stream<Item = Bytes> + Unpin + Send + 'static,
    {
        let server_node_message_sender = match &self.server_node_message_sender {
            Some(server_node_message_sender) => server_node_message_sender,
            None => return self.server.notify(module_id, body, origin).await,
        };

        let first = match body.next().await {
            None => return,
            Some(first) => first,
        };

        let mut previous = match body.next().await {
            None => {
                server_node_message_sender
                    .do_send_asynchronous(ServerNodeNotificationImmediateMessageInput::new(
                        module_id,
                        first,
                        origin.map(|o| {
                            ServerNodeNotificationOriginInput::new(
                                o.server_id(),
                                o.server_client_id(),
                            )
                        }),
                        None,
                    ))
                    .await;

                return;
            }
            Some(previous) => previous,
        };

        let (notification_id,) = server_node_message_sender
            .do_send_synchronous(ServerNodeNotificationStartMessageInput::new(
                module_id,
                first,
                origin.map(|o| {
                    ServerNodeNotificationOriginInput::new(o.server_id(), o.server_client_id())
                }),
                None,
            ))
            .await
            .into();

        loop {
            let next = match body.next().await {
                None => {
                    server_node_message_sender
                        .do_send_asynchronous(ServerNodeNotificationEndMessageInput::new(
                            notification_id,
                            previous,
                        ))
                        .await;

                    break;
                }
                Some(next) => next,
            };

            server_node_message_sender
                .do_send_asynchronous(ServerNodeNotificationNextMessageInput::new(
                    notification_id,
                    previous,
                ))
                .await;

            previous = next;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerServersServerClients<'a> {
    id: ServerId,
    server: &'a Server,
    server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
}

impl<'a> ServerServersServerClients<'a> {
    pub(crate) fn new(
        id: ServerId,
        server: &'a Server,
        server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
    ) -> Self {
        Self {
            id,
            server,
            server_node_message_sender,
        }
    }

    pub fn get(&'a self, server_client_id: ServerClientId) -> ServerServersServerClientsClient<'a> {
        ServerServersServerClientsClient::new(
            self.id,
            self.server,
            self.server_node_message_sender.clone(),
            server_client_id,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerServersServerClientsClient<'a> {
    id: ServerId,
    server: &'a Server,
    server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
    server_client_id: ServerClientId,
}

impl<'a> ServerServersServerClientsClient<'a> {
    pub(crate) fn new(
        id: ServerId,
        server: &'a Server,
        server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
        server_client_id: ServerClientId,
    ) -> Self {
        Self {
            id,
            server,
            server_node_message_sender,
            server_client_id,
        }
    }

    pub fn server_client_id(&self) -> ServerClientId {
        self.server_client_id
    }

    pub async fn notify<T>(&self, module_id: ModuleId, mut body: T)
    where
        T: Stream<Item = Bytes> + Unpin + Send + 'static,
    {
        if let Some(server_node_message_sender) = &self.server_node_message_sender {
            let first = match body.next().await {
                None => return,
                Some(first) => first,
            };

            let mut previous = match body.next().await {
                None => {
                    server_node_message_sender
                        .do_send_asynchronous(ServerNodeNotificationImmediateMessageInput::new(
                            module_id,
                            first,
                            None,
                            Some(ServerNotificationRedirection::new(
                                self.id,
                                Some(self.server_client_id),
                            )),
                        ))
                        .await;

                    return;
                }
                Some(previous) => previous,
            };

            let (notification_id,) = server_node_message_sender
                .do_send_synchronous(ServerNodeNotificationStartMessageInput::new(
                    module_id,
                    first,
                    None,
                    Some(ServerNotificationRedirection::new(
                        self.id,
                        Some(self.server_client_id),
                    )),
                ))
                .await
                .into();

            loop {
                let next = match body.next().await {
                    None => {
                        server_node_message_sender
                            .do_send_asynchronous(ServerNodeNotificationEndMessageInput::new(
                                notification_id,
                                previous,
                            ))
                            .await;

                        break;
                    }
                    Some(next) => next,
                };

                server_node_message_sender
                    .do_send_asynchronous(ServerNodeNotificationNextMessageInput::new(
                        notification_id,
                        previous,
                    ))
                    .await;

                previous = next;
            }
        } else {
            if let Some(client) = self.server.local_clients().get(self.server_client_id).await {
                client.notify(module_id, body).await;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerLogs<'a> {
    server: &'a Server,
}

impl<'a> ServerLogs<'a> {
    pub(crate) fn new(server: &'a Server) -> Self {
        Self { server }
    }

    pub(crate) fn server(&self) -> &Server {
        self.server
    }

    pub async fn create(&mut self, log_issuer: Option<LogIssuer>) -> LogId {
        let output = self
            .server
            .server_message_sender()
            .do_send_synchronous(ServerLogCreateMessageInput::new(log_issuer))
            .await;

        let (log_id,) = output.into();

        log_id
    }

    pub fn get(&self, log_id: LogId) -> ServerLogsLog<'_> {
        ServerLogsLog::new(log_id, self.server())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerLogsLog<'a> {
    log_id: LogId,
    server: &'a Server,
}

impl<'a> ServerLogsLog<'a> {
    pub(crate) fn new(log_id: LogId, server: &'a Server) -> Self {
        Self { log_id, server }
    }

    #[deprecated]
    pub async fn get(&self, log_entry_id: LogEntryId) -> Option<LogEntry> {
        let output = self
            .server
            .server_message_sender()
            .do_send_synchronous(ServerLogEntriesGetMessageInput::new(
                self.log_id,
                log_entry_id,
            ))
            .await;

        let (log_entry,) = output.into();

        log_entry
    }

    #[deprecated]
    pub async fn commit(&self, data: Bytes) {
        self.server
            .server_message_sender()
            .do_send_synchronized(ServerCommitMessageInput::new(self.log_id, data))
            .await;
    }

    pub fn entries(&self) -> ServerLogsLogEntries<'_> {
        ServerLogsLogEntries::new(self.log_id, self.server)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerLogsLogEntries<'a> {
    log_id: LogId,
    server: &'a Server,
}

impl<'a> ServerLogsLogEntries<'a> {
    pub(crate) fn new(log_id: LogId, server: &'a Server) -> Self {
        Self { log_id, server }
    }

    pub async fn get(&self, log_entry_id: LogEntryId) -> Option<LogEntry> {
        let output = self
            .server
            .server_message_sender()
            .do_send_synchronous(ServerLogEntriesGetMessageInput::new(
                self.log_id,
                log_entry_id,
            ))
            .await;

        let (log_entry,) = output.into();

        log_entry
    }

    pub async fn commit(&self, data: Bytes) {
        self.server
            .server_message_sender()
            .do_send_synchronized(ServerCommitMessageInput::new(self.log_id, data))
            .await;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerLocalClients<'a> {
    server: &'a Server,
}

impl<'a> ServerLocalClients<'a> {
    pub(crate) fn new(server: &'a Server) -> Self {
        Self { server }
    }

    pub fn server(&self) -> &Server {
        self.server
    }

    pub async fn get(
        &self,
        server_client_id: ServerClientId,
    ) -> Option<ServerLocalClientsLocalClient<'_>> {
        let output = self
            .server
            .server_message_sender()
            .do_send_synchronous(ServerClientGetMessageInput::new(server_client_id))
            .await;

        let (sender,) = output.into();

        sender
            .map(|sender| ServerLocalClientsLocalClient::new(server_client_id, self.server, sender))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerLocalClientsLocalClient<'a> {
    server_client_id: ServerClientId,
    server: &'a Server,
    server_client_message_sender: MessageQueueSender<ServerClientMessage>,
}

impl<'a> ServerLocalClientsLocalClient<'a> {
    pub(crate) fn new(
        server_client_id: ServerClientId,
        server: &'a Server,
        server_client_message_sender: MessageQueueSender<ServerClientMessage>,
    ) -> Self {
        Self {
            server_client_id,
            server,
            server_client_message_sender,
        }
    }

    pub fn server_client_id(&self) -> ServerClientId {
        self.server_client_id
    }

    pub fn server(&self) -> &Server {
        self.server
    }

    pub async fn notify<T>(&self, module_id: ModuleId, mut body: T)
    where
        T: Stream<Item = Bytes> + Unpin + Send + 'static,
    {
        let first = match body.next().await {
            None => return,
            Some(first) => first,
        };

        let mut previous = match body.next().await {
            None => {
                self.server_client_message_sender
                    .do_send_asynchronous(ServerClientNotificationImmediateMessageInput::new(
                        module_id, first,
                    ))
                    .await;

                return;
            }
            Some(previous) => previous,
        };

        let (notification_id,) = self
            .server_client_message_sender
            .do_send_synchronous(ServerClientNotificationStartMessageInput::new(
                module_id, first,
            ))
            .await
            .into();

        loop {
            let next = match body.next().await {
                None => {
                    self.server_client_message_sender
                        .do_send_asynchronous(ServerClientNotificationEndMessageInput::new(
                            notification_id,
                            previous,
                        ))
                        .await;

                    break;
                }
                Some(next) => next,
            };

            self.server_client_message_sender
                .do_send_asynchronous(ServerClientNotificationNextMessageInput::new(
                    notification_id,
                    previous,
                ))
                .await;

            previous = next;
        }
    }

    pub async fn send(&self, bytes: Bytes) {
        self.server_client_message_sender
            .do_send_asynchronous(ServerClientSendMessageInput::new(bytes))
            .await;
    }
}
