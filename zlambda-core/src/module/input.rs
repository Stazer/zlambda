use crate::server::ServerMessage;
use crate::message::MessageQueueSender;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleManager {
    sender: MessageQueueSender<ServerMessage>,
}

impl ModuleManager {
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Server {
    sender: MessageQueueSender<ServerMessage>,
    module_manager: ModuleManager,
}

impl Server {
    pub fn new(sender: MessageQueueSender<ServerMessage>) -> Self {
        Self {
            sender: sender.clone(),
            module_manager: ModuleManager {
                sender,
            }
        }
    }

    pub fn module_manager(&self) -> &ModuleManager {
        &self.module_manager
    }

    pub fn module_manager_mut(&mut self) -> &mut ModuleManager {
        &mut self.module_manager
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleStartupEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleShutdownEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleInitializeEventInput {
    server: Server,
}

impl From<ModuleInitializeEventInput> for (Server,) {
    fn from(input: ModuleInitializeEventInput) -> Self {
        (input.server,)
    }
}

impl ModuleInitializeEventInput {
    pub fn new(
        server: Server,
    ) -> Self {
        Self {
            server,
        }
    }

    pub fn server(&self) -> &Server {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut Server {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleFinalizeEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleDispatchEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleCommitEventInput = ();
