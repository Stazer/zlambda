use crate::common::module::ModuleId;
use crate::common::utility::Bytes;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ServerClientShutdownMessageInput {}

impl From<ServerClientShutdownMessageInput> for () {
    fn from(_input: ServerClientShutdownMessageInput) -> Self {
        ()
    }
}

impl ServerClientShutdownMessageInput {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerClientNotificationImmediateMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ServerClientNotificationImmediateMessageInput> for (ModuleId, Bytes) {
    fn from(input: ServerClientNotificationImmediateMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ServerClientNotificationImmediateMessageInput {
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
pub struct ServerClientNotificationStartMessageInput {
    module_id: ModuleId,
    body: Bytes,
}

impl From<ServerClientNotificationStartMessageInput> for (ModuleId, Bytes) {
    fn from(input: ServerClientNotificationStartMessageInput) -> Self {
        (input.module_id, input.body)
    }
}

impl ServerClientNotificationStartMessageInput {
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
pub struct ServerClientNotificationNextMessageInput {
    notification_id: usize,
    body: Bytes,
}

impl From<ServerClientNotificationNextMessageInput> for (usize, Bytes) {
    fn from(input: ServerClientNotificationNextMessageInput) -> Self {
        (input.notification_id, input.body)
    }
}

impl ServerClientNotificationNextMessageInput {
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
pub struct ServerClientNotificationEndMessageInput {
    notification_id: usize,
    body: Bytes,
}

impl From<ServerClientNotificationEndMessageInput> for (usize, Bytes) {
    fn from(input: ServerClientNotificationEndMessageInput) -> Self {
        (input.notification_id, input.body)
    }
}

impl ServerClientNotificationEndMessageInput {
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
