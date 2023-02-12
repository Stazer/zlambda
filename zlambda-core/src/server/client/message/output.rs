#[derive(Debug)]
pub struct ServerClientNotificationStartMessageOutput {
    notification_id: usize,
}

impl From<ServerClientNotificationStartMessageOutput> for (usize,) {
    fn from(output: ServerClientNotificationStartMessageOutput) -> Self {
        (output.notification_id,)
    }
}

impl ServerClientNotificationStartMessageOutput {
    pub fn new(notification_id: usize) -> Self {
        Self { notification_id }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }
}
