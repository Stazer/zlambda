use crate::module::{
    ModuleCommitEventInput, ModuleCommitEventOutput, ModuleDispatchEventInput,
    ModuleDispatchEventOutput, ModuleInitializeEventInput,
    ModuleInitializeEventOutput, ModuleShutdownEventInput,
    ModuleShutdownEventOutput, ModuleStartupEventInput,
    ModuleStartupEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Module {
    async fn on_startup(_event: ModuleStartupEventInput) -> ModuleStartupEventOutput {
    }

    async fn on_shutdown(
        _event: ModuleShutdownEventInput,
    ) -> ModuleShutdownEventOutput {
    }

    async fn on_initialize(
        _event: ModuleInitializeEventInput,
    ) -> ModuleInitializeEventOutput {
    }

    async fn on_finalize(
        _event: ModuleInitializeEventInput,
    ) -> ModuleInitializeEventOutput {
    }

    async fn on_dispach(
        &self,
        _event: ModuleDispatchEventInput,
    ) -> ModuleDispatchEventOutput {
    }

    async fn on_commit(
        &self,
        _event: ModuleCommitEventInput,
    ) -> ModuleCommitEventOutput {
    }
}
