use crate::module::ModuleId;
use crate::server::ServerHandle;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleStartupEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleShutdownEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ModuleInitializeEventInput {
    module_id: ModuleId,
    server: ServerHandle,
}

impl From<ModuleInitializeEventInput> for (ModuleId, ServerHandle) {
    fn from(input: ModuleInitializeEventInput) -> Self {
        (input.module_id, input.server)
    }
}

impl ModuleInitializeEventInput {
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

pub struct ModuleFinalizeEventInput {
    server: ServerHandle,
}

impl From<ModuleFinalizeEventInput> for (ServerHandle,) {
    fn from(input: ModuleFinalizeEventInput) -> Self {
        (input.server,)
    }
}

impl ModuleFinalizeEventInput {
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
