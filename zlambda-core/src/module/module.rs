use crate::module::{
    ModuleCommitEventInput, ModuleCommitEventOutput, ModuleDispatchEventInput,
    ModuleDispatchEventOutput, ModuleUnloadEventInput, ModuleUnloadEventOutput,
    ModuleLoadEventInput, ModuleLoadEventOutput, ModuleShutdownEventInput,
    ModuleShutdownEventOutput, ModuleStartupEventInput, ModuleStartupEventOutput,
};
use std::any::Any;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
pub trait Module: Any + Send + Sync + 'static {
    async fn on_startup(&self, _event: ModuleStartupEventInput) -> ModuleStartupEventOutput {}

    async fn on_shutdown(&self, _event: ModuleShutdownEventInput) -> ModuleShutdownEventOutput {}

    async fn on_load(
        &self,
        _event: ModuleLoadEventInput,
    ) -> ModuleLoadEventOutput {
    }

    async fn on_unload(&self, _event: ModuleUnloadEventInput) -> ModuleUnloadEventOutput {}

    async fn on_dispatch(&self, _event: ModuleDispatchEventInput) -> ModuleDispatchEventOutput {}

    async fn on_commit(&self, _event: ModuleCommitEventInput) -> ModuleCommitEventOutput {}
}
