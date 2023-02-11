use crate::common::module::ModuleId;
use crate::common::utility::Bytes;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientNotificationStartMessageInput {
    module_id: ModuleId,
}

impl From<ClientNotificationStartMessageInput> for (ModuleId,) {
    fn from(input: ClientNotificationStartMessageInput) -> Self {
        (input.module_id,)
    }
}

impl ClientNotificationStartMessageInput {
    pub fn new(
        module_id: ModuleId,
    ) -> Self {
        Self {
            module_id,
        }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientNotificationNextMessageInput {
    notification_id: usize,
    body: Option<Bytes>,
}

impl From<ClientNotificationNextMessageInput> for (usize, Option<Bytes>) {
    fn from(input: ClientNotificationNextMessageInput) -> Self {
        (input.notification_id, input.body)
    }
}

impl ClientNotificationNextMessageInput {
    pub fn new(
        notification_id: usize,
        body: Option<Bytes>,
    ) -> Self {
        Self {
            notification_id,
            body,
        }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }

    pub fn body(&self) -> &Option<Bytes> {
        &self.body
    }
}
