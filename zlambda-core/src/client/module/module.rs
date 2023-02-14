use crate::client::{
    ClientModuleFinalizeEventInput, ClientModuleFinalizeEventOutput,
    ClientModuleInitializeEventInput, ClientModuleInitializeEventOutput,
    ClientModuleNotificationEventInput, ClientModuleNotificationEventOutput,
};
use crate::common::module::Module;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
pub trait ClientModule: Module {
    async fn on_initialize(
        &self,
        _event: ClientModuleInitializeEventInput,
    ) -> ClientModuleInitializeEventOutput {
    }

    async fn on_finalize(
        &self,
        _event: ClientModuleFinalizeEventInput,
    ) -> ClientModuleFinalizeEventOutput {
    }

    async fn on_notification(
        &self,
        _event: ClientModuleNotificationEventInput,
    ) -> ClientModuleNotificationEventOutput {
    }
}
