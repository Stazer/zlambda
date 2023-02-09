use crate::common::module::ModuleId;
use crate::common::utility::Bytes;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientNotifyMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ClientNotifyMessageInput> for (ModuleId, Bytes) {
    fn from(input: ClientNotifyMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ClientNotifyMessageInput {
    pub fn new(module_id: ModuleId, body: Bytes) -> Self {
        Self { module_id, body }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}
