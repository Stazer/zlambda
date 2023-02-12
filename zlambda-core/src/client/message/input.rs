use crate::common::module::ModuleId;
use crate::common::utility::Bytes;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientNotificationImmediateMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ClientNotificationImmediateMessageInput> for (ModuleId, Bytes) {
    fn from(input: ClientNotificationImmediateMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ClientNotificationImmediateMessageInput {
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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientNotificationStartMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ClientNotificationStartMessageInput> for (ModuleId, Bytes) {
    fn from(input: ClientNotificationStartMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ClientNotificationStartMessageInput {
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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientNotificationNextMessageInput {
    notification_id: usize,
    body: Bytes,
}

impl From<ClientNotificationNextMessageInput> for (usize, Bytes) {
    fn from(input: ClientNotificationNextMessageInput) -> Self {
        (input.notification_id, input.body)
    }
}

impl ClientNotificationNextMessageInput {
    pub fn new(notification_id: usize, body: Bytes) -> Self {
        Self {
            notification_id,
            body,
        }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientNotificationEndMessageInput {
    notification_id: usize,
    body: Bytes,
}

impl From<ClientNotificationEndMessageInput> for (usize, Bytes) {
    fn from(input: ClientNotificationEndMessageInput) -> Self {
        (input.notification_id, input.body)
    }
}

impl ClientNotificationEndMessageInput {
    pub fn new(notification_id: usize, body: Bytes) -> Self {
        Self {
            notification_id,
            body,
        }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}
