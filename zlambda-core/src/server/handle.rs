use crate::common::message::MessageQueueSender;
use crate::common::module::ModuleId;
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
}

impl ServerHandle {
    pub(crate) fn new(sender: MessageQueueSender<ServerMessage>) -> Self {
        Self {
            sender: sender.clone(),
            module_manager: ServerModuleManagerHandle::new(sender.clone()),
            log_manager: ServerLogManagerHandle::new(sender.clone()),
            node_manager: ServerNodeManagerHandle::new(sender),
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

    pub async fn load(&mut self, module: Arc<dyn ServerModule>) {
        self.sender
            .do_send_synchronous(ServerModuleLoadMessageInput::new(module))
            .await;
    }

    pub async fn unload(&mut self, module_id: ModuleId) {
        self.sender
            .do_send_synchronous(ServerModuleUnloadMessageInput::new(module_id))
            .await;
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
