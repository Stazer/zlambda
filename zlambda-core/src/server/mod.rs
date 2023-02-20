mod client;
mod connection;
mod error;
mod id;
mod log;
mod message;
mod module;
mod node;
mod task;
mod r#type;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use client::*;
pub use connection::*;
pub use error::*;
pub use id::*;
pub use log::*;
pub use message::*;
pub use module::*;
pub use node::*;
pub use r#type::*;
pub use task::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::message::message_queue;
use crate::common::message::MessageQueueSender;
use crate::common::module::{LoadModuleError, ModuleId, UnloadModuleError};
use crate::common::net::ToSocketAddrs;
use crate::common::runtime::spawn;
use crate::common::utility::Bytes;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::future::pending;
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

    pub fn nodes(&self) -> ServerNodes<'_> {
        ServerNodes::new(self)
    }

    pub async fn notify<T>(&self, module_id: ModuleId, mut body: T)
    where
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

        let (sender, receiver) = message_queue();
        let server_source =
            ServerModuleNotificationEventInputServerSource::new(self.server_id().await);

        module
            .on_notification(ServerModuleNotificationEventInput::new(
                //self.clone(),
                Server::new(self.server_message_sender.clone()),
                server_source.into(),
                ServerModuleNotificationEventBody::new(receiver),
            ))
            .await;

        spawn(async move {
            while let Some(body) = body.next().await {
                sender.do_send(body).await;
            }
        });
    }

    pub async fn commit(&self, data: Bytes) {
        self.server_message_sender
            .do_send_synchronized(ServerCommitMessageInput::new(data))
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerNodes<'a> {
    server: &'a Server,
}

impl<'a> ServerNodes<'a> {
    pub(crate) fn new(server: &'a Server) -> Self {
        Self { server }
    }

    pub(crate) fn server(&self) -> &Server {
        self.server
    }

    pub async fn get(&self, server_id: ServerId) -> Option<ServerNodesNode<'_>> {
        if self.server().server_id().await == server_id {
            Some(ServerNodesNode::new(self.server(), None))
        } else {
            let output = self
                .server
                .server_message_sender()
                .do_send_synchronous(ServerServerNodeMessageSenderGetMessageInput::new(server_id))
                .await;

            let (server_node_message_sender,) = output.into();

            server_node_message_sender.map(|server_node_message_sender| {
                ServerNodesNode::new(self.server(), Some(server_node_message_sender))
            })
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerNodesNode<'a> {
    server: &'a Server,
    server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
}

impl<'a> ServerNodesNode<'a> {
    pub(crate) fn new(
        server: &'a Server,
        server_node_message_sender: Option<MessageQueueSender<ServerNodeMessage>>,
    ) -> Self {
        Self {
            server,
            server_node_message_sender,
        }
    }

    pub async fn notify<T>(&self, module_id: ModuleId, mut body: T)
    where
        T: Stream<Item = Bytes> + Unpin + Send + 'static,
    {
        let server_node_message_sender = match &self.server_node_message_sender {
            Some(server_node_message_sender) => server_node_message_sender,
            None => return self.server.notify(module_id, body).await,
        };

        let first = match body.next().await {
            None => return,
            Some(first) => first,
        };

        let mut previous = match body.next().await {
            None => {
                server_node_message_sender
                    .do_send_asynchronous(ServerNodeNotificationImmediateMessageInput::new(
                        module_id, first,
                    ))
                    .await;

                return;
            }
            Some(previous) => previous,
        };

        let (notification_id,) = server_node_message_sender
            .do_send_synchronous(ServerNodeNotificationStartMessageInput::new(
                module_id, first,
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
