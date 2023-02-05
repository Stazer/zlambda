use crate::common::module::ModuleId;
use crate::server::{ServerHandle, LogEntry};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerModuleStartupEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerModuleShutdownEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
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

pub type ServerModuleDispatchEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerModuleCommitEventInput {
    server: ServerHandle,
    log_entry: LogEntry,
}

impl ServerModuleCommitEventInput {
    pub fn new(
        server: ServerHandle,
        log_entry: LogEntry,
    ) -> Self {
        Self {
            server,
            log_entry,
        }
    }

    pub fn server(&self) -> &ServerHandle {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut ServerHandle {
        &mut self.server
    }

    pub fn log_entry(&self) -> &LogEntry {
        &self.log_entry
    }

    pub fn log_entry_mut(&mut self) -> &mut LogEntry {
        &mut self.log_entry
    }
}
