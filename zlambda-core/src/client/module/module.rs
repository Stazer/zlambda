use crate::client::{ClientModuleNotifyEventInput, ClientModuleNotifyEventOutput};
use crate::common::module::Module;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
pub trait ClientModule: Module {
    async fn on_notify(
        &self,
        _event: ClientModuleNotifyEventInput,
    ) -> ClientModuleNotifyEventOutput;
}
