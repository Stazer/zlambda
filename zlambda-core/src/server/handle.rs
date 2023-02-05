use crate::message::MessageQueueSender;
use crate::server::{ServerModuleGetMessageInput, ServerModuleLoadMessageInput, ServerModuleUnloadMessageInput, ServerMessage, ServerModule};
use crate::common::module::{ModuleId};
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ServerHandle {
    sender: MessageQueueSender<ServerMessage>,
    module_manager: ServerModuleManagerHandle,
}

impl ServerHandle {
    pub(crate) fn new(sender: MessageQueueSender<ServerMessage>) -> Self {
        Self {
            sender: sender.clone(),
            module_manager: ServerModuleManagerHandle::new(sender),
        }
    }

    pub fn module_manager(&self) -> &ServerModuleManagerHandle {
        &self.module_manager
    }

    pub fn module_manager_mut(&mut self) -> &mut ServerModuleManagerHandle {
        &mut self.module_manager
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
        let (module, ) = self.sender.do_send_synchronous(ServerModuleGetMessageInput::new(module_id)).await.into();

        module
    }

    pub async fn load(&mut self, module: Arc<dyn ServerModule>) {
        self.sender.do_send_synchronous(ServerModuleLoadMessageInput::new(module)).await;
    }

    pub async fn unload(&mut self, module_id: ModuleId) {
        self.sender.do_send_synchronous(ServerModuleUnloadMessageInput::new(module_id)).await;
    }
}
