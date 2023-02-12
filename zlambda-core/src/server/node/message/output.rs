#[derive(Debug)]
pub struct ServerNodeNotificationStartMessageOutput {
    notification_id: usize,
}

impl From<ServerNodeNotificationStartMessageOutput> for (usize,) {
    fn from(output: ServerNodeNotificationStartMessageOutput) -> Self {
        (output.notification_id,)
    }
}

impl ServerNodeNotificationStartMessageOutput {
    pub fn new(notification_id: usize) -> Self {
        Self { notification_id }
    }

    pub fn notification_id(&self) -> usize {
        self.notification_id
    }
}
