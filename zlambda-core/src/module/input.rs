use crate::server::ServerHandle;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleStartupEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleShutdownEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ModuleInitializeEventInput {
    server: ServerHandle,
}

impl From<ModuleInitializeEventInput> for (ServerHandle,) {
    fn from(input: ModuleInitializeEventInput) -> Self {
        (input.server,)
    }
}

impl ModuleInitializeEventInput {
    pub fn new(server: ServerHandle) -> Self {
        Self {
            server,
        }
    }

    pub fn server(&self) -> &ServerHandle {
        &self.server
    }

    pub fn server_mut(&mut self) -> &mut ServerHandle {
        &mut self.server
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleFinalizeEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleDispatchEventInput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleCommitEventInput = ();