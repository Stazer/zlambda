use crate::module::ModuleId;
use crate::server::ServerHandle;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleStartupEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleShutdownEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ModuleLoadEventInput {
    module_id: ModuleId,
    server: ServerHandle,
}

impl From<ModuleLoadEventInput> for (ModuleId, ServerHandle) {
    fn from(input: ModuleLoadEventInput) -> Self {
        (input.module_id, input.server)
    }
}

impl ModuleLoadEventInput {
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

pub struct ModuleUnloadEventInput {
    server: ServerHandle,
}

impl From<ModuleUnloadEventInput> for (ServerHandle,) {
    fn from(input: ModuleUnloadEventInput) -> Self {
        (input.server,)
    }
}

impl ModuleUnloadEventInput {
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

pub type ModuleDispatchEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleCommitEventInput = ();
