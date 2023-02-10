use crate::common::module::Module;
use crate::server::{
    ServerModuleCommitEventInput, ServerModuleCommitEventOutput, ServerModuleLoadEventInput,
    ServerModuleLoadEventOutput, ServerModuleNotifyEventInput, ServerModuleNotifyEventOutput,
    ServerModuleShutdownEventInput, ServerModuleShutdownEventOutput, ServerModuleStartupEventInput,
    ServerModuleStartupEventOutput, ServerModuleUnloadEventInput, ServerModuleUnloadEventOutput,
    ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
pub trait ServerModule: Module {
    async fn on_startup(
        &self,
        _event: ServerModuleStartupEventInput,
    ) -> ServerModuleStartupEventOutput {
    }

    async fn on_shutdown(
        &self,
        _event: ServerModuleShutdownEventInput,
    ) -> ServerModuleShutdownEventOutput {
    }

    async fn on_load(&self, _event: ServerModuleLoadEventInput) -> ServerModuleLoadEventOutput {}

    async fn on_unload(
        &self,
        _event: ServerModuleUnloadEventInput,
    ) -> ServerModuleUnloadEventOutput {
    }

    async fn on_notify(
        &self,
        _event: ServerModuleNotifyEventInput,
    ) -> ServerModuleNotifyEventOutput {
    }

    async fn on_notification(
        &self,
        _event: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
    }

    async fn on_commit(
        &self,
        _event: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
    }
}
