mod input;
mod output;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use input::*;
pub use output::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::module::Module;

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
