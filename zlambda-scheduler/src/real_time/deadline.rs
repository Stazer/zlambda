use zlambda_core::common::module::{Module};
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};
use zlambda_core::common::async_trait;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct DeadlineRealTimeScheduler {
}

#[async_trait]
impl Module for DeadlineRealTimeScheduler {}

#[async_trait]
impl ServerModule for DeadlineRealTimeScheduler {
    async fn on_notification(
        &self,
        _input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
    }
}
