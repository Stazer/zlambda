use zlambda_core::common::async_trait;
use zlambda_core::common::module::{Module};
use serde::{Deserialize, Serialize};
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
enum LogEntryData {
    Prepare(usize),
    Append(usize),
    Load(usize),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
struct NotificationHeader {

}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct EbpfLoader {}

#[async_trait]
impl Module for EbpfLoader {}

#[async_trait]
impl ServerModule for EbpfLoader {
    async fn on_notification(
        &self,
        _input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
    }
}