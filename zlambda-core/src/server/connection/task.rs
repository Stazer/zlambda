use crate::general::GeneralMessage;
use crate::message::{MessageQueueSender, MessageSocketReceiver, MessageSocketSender};
use crate::server::ServerMessage;
use tokio::spawn;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerConnectionTask {
    server_sender: MessageQueueSender<ServerMessage>,
    general_sender: MessageSocketSender<GeneralMessage>,
    general_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl ServerConnectionTask {
    pub fn new(
        server_sender: MessageQueueSender<ServerMessage>,
        general_sender: MessageSocketSender<GeneralMessage>,
        general_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            server_sender,
            general_sender,
            general_receiver,
        }
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await })
    }

    pub async fn run(self) {}
}
