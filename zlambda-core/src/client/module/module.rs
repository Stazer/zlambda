use crate::client::{ClientModuleNotificationEventOutput, ClientModuleNotificationEventInput};
use crate::common::module::Module;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
pub trait ClientModule: Module {
    async fn on_notification(
        &self,
        _event: ClientModuleNotificationEventInput,
    ) -> ClientModuleNotificationEventOutput;
}
