use crate::common::module::ModuleId;
use crate::common::utility::Bytes;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerClientNotifyMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ServerClientNotifyMessageInput> for (ModuleId, Bytes) {
    fn from(input: ServerClientNotifyMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ServerClientNotifyMessageInput {
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
