#[derive(Debug)]
pub struct ClientNotificationStartMessageOutput {
    notification_id: usize,
}

impl From<ClientNotificationStartMessageOutput> for (usize,) {
    fn from(output: ClientNotificationStartMessageOutput) -> Self {
        (output.notification_id,)
    }
}

impl ClientNotificationStartMessageOutput {
    pub fn new(notification_id: usize) -> Self {
        Self { notification_id }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }
}
