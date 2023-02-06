use crate::common::message::MessageQueueSender;
use crate::common::module::{LoadModuleError, ModuleId, UnloadModuleError};
use crate::server::{
    ServerMessage, ServerModule, ServerModuleGetMessageInput, ServerModuleLoadMessageInput,
    ServerModuleUnloadMessageInput,
};
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ServerHandle {
    sender: MessageQueueSender<ServerMessage>,
    module_manager: ServerModuleManagerHandle,
    log_manager: ServerLogManagerHandle,
    node_manager: ServerNodeManagerHandle,
    client_manager: ServerClientManagerHandle,
}

impl ServerHandle {
    pub(crate) fn new(sender: MessageQueueSender<ServerMessage>) -> Self {
        Self {
            sender: sender.clone(),
            module_manager: ServerModuleManagerHandle::new(sender.clone()),
            log_manager: ServerLogManagerHandle::new(sender.clone()),
            node_manager: ServerNodeManagerHandle::new(sender.clone()),
            client_manager: ServerClientManagerHandle::new(sender),
        }
    }

    pub fn module_manager(&self) -> &ServerModuleManagerHandle {
        &self.module_manager
    }

    pub fn module_manager_mut(&mut self) -> &mut ServerModuleManagerHandle {
        &mut self.module_manager
    }

    pub fn log_manager(&self) -> &ServerLogManagerHandle {
        &self.log_manager
    }

    pub fn log_manager_mut(&mut self) -> &mut ServerLogManagerHandle {
        &mut self.log_manager
    }

    pub fn node_manager(&self) -> &ServerNodeManagerHandle {
        &self.node_manager
    }

    pub fn node_manager_mut(&mut self) -> &mut ServerNodeManagerHandle {
        &mut self.node_manager
    }

    pub fn client_manager(&self) -> &ServerClientManagerHandle {
        &self.client_manager
    }

    pub fn client_manager_mut(&mut self) -> &mut ServerClientManagerHandle {
        &mut self.client_manager
    }

    pub async fn ping(&self) {
        self.sender.do_send(ServerMessage::Ping).await;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ServerModuleManagerHandle {
    sender: MessageQueueSender<ServerMessage>,
}

impl ServerModuleManagerHandle {
    pub(crate) fn new(sender: MessageQueueSender<ServerMessage>) -> Self {
        Self { sender }
    }

    pub async fn get(&self, module_id: ModuleId) -> Option<Arc<dyn ServerModule>> {
        let (module,) = self
            .sender
            .do_send_synchronous(ServerModuleGetMessageInput::new(module_id))
            .await
            .into();

        module
    }

    pub async fn load(
        &mut self,
        module: Arc<dyn ServerModule>,
    ) -> Result<ModuleId, LoadModuleError> {
        let (result,) = self
            .sender
            .do_send_synchronous(ServerModuleLoadMessageInput::new(module))
            .await
            .into();

        result
    }

    pub async fn unload(&mut self, module_id: ModuleId) -> Result<(), UnloadModuleError> {
        let (result,) = self
            .sender
            .do_send_synchronous(ServerModuleUnloadMessageInput::new(module_id))
            .await
            .into();

        result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ServerLogManagerHandle {
    sender: MessageQueueSender<ServerMessage>,
}

impl ServerLogManagerHandle {
    pub(crate) fn new(sender: MessageQueueSender<ServerMessage>) -> Self {
        Self { sender }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ServerNodeManagerHandle {
    sender: MessageQueueSender<ServerMessage>,
}

impl ServerNodeManagerHandle {
    pub(crate) fn new(sender: MessageQueueSender<ServerMessage>) -> Self {
        Self { sender }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ServerClientManagerHandle {
    sender: MessageQueueSender<ServerMessage>,
}

impl ServerClientManagerHandle {
    pub(crate) fn new(sender: MessageQueueSender<ServerMessage>) -> Self {
        Self { sender }
    }
}
