use crate::general::GeneralMessage;
use crate::message::{
    message_queue, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::server::member::ServerMemberMessage;
use crate::server::{ServerId, ServerMessage};
use tokio::spawn;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerMemberTask {
    server_id: ServerId,
    server_sender: MessageQueueSender<ServerMessage>,
    general_sender: Option<MessageSocketSender<GeneralMessage>>,
    general_receiver: Option<MessageSocketReceiver<GeneralMessage>>,
    sender: MessageQueueSender<ServerMemberMessage>,
    receiver: MessageQueueReceiver<ServerMemberMessage>,
}

impl ServerMemberTask {
    pub fn new(
        server_id: ServerId,
        server_sender: MessageQueueSender<ServerMessage>,
        general_sender: Option<MessageSocketSender<GeneralMessage>>,
        general_receiver: Option<MessageSocketReceiver<GeneralMessage>>,
    ) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            server_id,
            server_sender,
            general_sender,
            general_receiver,
            sender,
            receiver,
        }
    }

    pub fn sender(&self) -> &MessageQueueSender<ServerMemberMessage> {
        &self.sender
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(self) {
        loop {}
    }
}
