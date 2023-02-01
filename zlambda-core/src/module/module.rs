use std::any::Any;
use crate::module::{
    ModuleCommitEventInput, ModuleCommitEventOutput, ModuleDispatchEventInput,
    ModuleDispatchEventOutput, ModuleInitializeEventInput,
    ModuleInitializeEventOutput, ModuleShutdownEventInput,
    ModuleShutdownEventOutput, ModuleStartupEventInput,
    ModuleStartupEventOutput,ModuleFinalizeEventInput,
    ModuleFinalizeEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
pub trait Module: Any + Send + Sync + 'static {
    async fn on_startup(&self, _event: ModuleStartupEventInput) -> ModuleStartupEventOutput
    {
    }

    async fn on_shutdown(
        &self,
        _event: ModuleShutdownEventInput,
    ) -> ModuleShutdownEventOutput
    {
    }

    async fn on_initialize(
        &self,
        _event: ModuleInitializeEventInput,
    ) -> ModuleInitializeEventOutput
    {
    }

    async fn on_finalize(
        &self,
        _event: ModuleFinalizeEventInput,
    ) -> ModuleFinalizeEventOutput
    {
    }

    async fn on_dispach(
        &self,
        _event: ModuleDispatchEventInput,
    ) -> ModuleDispatchEventOutput
    {
    }

    async fn on_commit(
        &self,
        _event: ModuleCommitEventInput,
    ) -> ModuleCommitEventOutput
    {
    }
}
