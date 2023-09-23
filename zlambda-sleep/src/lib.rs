use std::time::Duration;
use serde::Deserialize;
use zlambda_core::common::async_trait;
use zlambda_core::common::module::Module as CommonModule;
use zlambda_core::common::time::sleep;
use zlambda_core::common::notification::{
    NotificationBodyItemStreamExt,
};
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
struct NotificationHeader {
    seconds: u64,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct SleepModule {}

#[async_trait]
impl CommonModule for SleepModule {}

#[async_trait]
impl ServerModule for SleepModule {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (_server, _source, notification_body_item_queue_receiver) = input.into();
        let mut deserializer = notification_body_item_queue_receiver.deserializer();

        let header = deserializer
            .deserialize::<NotificationHeader>()
            .await
            .unwrap();

        sleep(Duration::from_secs(header.seconds)).await;
    }
}
