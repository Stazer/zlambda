use crate::message::MessageQueueSender;
use crate::server::ServerMessage;
use crate::common::module::{ModuleId, Module};
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

    pub fn get(&self, module_id: ModuleId) -> Option<Arc<dyn Module>> {
        None
    }
}
