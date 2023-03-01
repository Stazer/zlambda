use libloading::Library;
use std::collections::HashMap;
use zlambda_core::common::async_trait;
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::sync::Mutex;
use zlambda_core::server::{
    self, ServerModuleCommitEventInput, ServerModuleCommitEventOutput,
    ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct DynamicLibraryManager {
    _libraries: Mutex<HashMap<ModuleId, Library>>,
}

#[async_trait]
impl Module for DynamicLibraryManager {}

#[async_trait]
impl server::ServerModule for DynamicLibraryManager {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (_server, _source, mut notification_body_item_queue_receiver) = input.into();

        while let Some(bytes) = notification_body_item_queue_receiver.next().await {
            println!("{:?}", bytes);
        }
    }

    async fn on_commit(
        &self,
        _input: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
    }
}
