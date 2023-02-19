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

use crate::common::message::MessageQueueSender;
use crate::common::module::{LoadModuleError, ModuleId, UnloadModuleError};
use crate::common::net::ToSocketAddrs;
use crate::common::utility::Bytes;
use std::fmt::Debug;
use std::future::pending;
use std::marker::PhantomData;
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

#[derive(Clone, Debug)]
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

    pub fn modules(&self) -> ServerModules<'_> {
        ServerModules::new(&self.server_message_sender)
    }

    pub fn nodes(&self) -> ServerNodes<'_> {
        ServerNodes::new(&self.server_message_sender)
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
    server_message_sender: &'a MessageQueueSender<ServerMessage>,
}

impl<'a> ServerModules<'a> {
    pub(crate) fn new(server_message_sender: &'a MessageQueueSender<ServerMessage>) -> Self {
        Self {
            server_message_sender,
        }
    }

    pub async fn load<T>(&self, module: T) -> Result<ModuleId, LoadModuleError>
    where
        T: IntoArcServerModule,
    {
        let output = self
            .server_message_sender
            .do_send_synchronous(ServerModuleLoadMessageInput::new(
                module.into_arc_server_module(),
            ))
            .await;

        let (result,) = output.into();

        result
    }

    pub async fn unload(&self, module_id: ModuleId) -> Result<(), UnloadModuleError> {
        let output = self
            .server_message_sender
            .do_send_synchronous(ServerModuleUnloadMessageInput::new(module_id))
            .await;

        let (result,) = output.into();

        result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerNodes<'a> {
    server_message_sender: &'a MessageQueueSender<ServerMessage>,
}

impl<'a> ServerNodes<'a> {
    pub(crate) fn new(server_message_sender: &'a MessageQueueSender<ServerMessage>) -> Self {
        Self {
            server_message_sender,
        }
    }

    pub async fn get(&self, server_id: ServerId) -> Option<ServerNodesNode<'_>> {
        let output = self
            .server_message_sender
            .do_send_synchronous(ServerServerNodeMessageSenderGetMessageInput::new(server_id))
            .await;

        let (server_node_message_sender,) = output.into();

        server_node_message_sender.map(ServerNodesNode::new)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerNodesNode<'a> {
    server_node_message_sender: MessageQueueSender<ServerNodeMessage>,
    phantom: PhantomData<&'a MessageQueueSender<ServerNodeMessage>>,
}

impl<'a> ServerNodesNode<'a> {
    pub(crate) fn new(server_node_message_sender: MessageQueueSender<ServerNodeMessage>) -> Self {
        Self {
            server_node_message_sender,
            phantom: PhantomData,
        }
    }
}
