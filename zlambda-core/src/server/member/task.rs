use crate::general::GeneralMessage;
use crate::message::{
    message_queue, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::server::member::{ServerMemberMessage, ServerMemberReplicationMessage, ServerMemberRegistrationMessage};
use crate::server::{ServerId, ServerMessage};
use tokio::{spawn, select};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerMemberTask {
    server_id: ServerId,
    server_queue_sender: MessageQueueSender<ServerMessage>,
    general_socket_sender: Option<MessageSocketSender<GeneralMessage>>,
    general_socket_receiver: Option<MessageSocketReceiver<GeneralMessage>>,
    sender: MessageQueueSender<ServerMemberMessage>,
    receiver: MessageQueueReceiver<ServerMemberMessage>,
}

impl ServerMemberTask {
    pub fn new(
        server_id: ServerId,
        server_queue_sender: MessageQueueSender<ServerMessage>,
        general_socket_sender: Option<MessageSocketSender<GeneralMessage>>,
        general_socket_receiver: Option<MessageSocketReceiver<GeneralMessage>>,
    ) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            server_id,
            server_queue_sender,
            general_socket_sender,
            general_socket_receiver,
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

    pub async fn run(mut self) {
        loop {
            self.select().await
        }
    }

    async fn select(&mut self)  {
        match &mut self.general_socket_receiver {
            Some(ref mut general_socket_receiver) => {
                select!(
                    result = general_socket_receiver.receive() => {

                    }
                    result = self.receiver.do_receive() => {

                    }
                )
            }
            None => {
                select!(
                    result = self.receiver.do_receive() => {

                    }
                )
            }
        }
    }

    async fn on_server_member_message(&mut self, message: ServerMemberMessage) {
        match message {
            ServerMemberMessage::Replication(message) => self.on_server_member_replication_message(message).await,
            ServerMemberMessage::Registration(message) => self.on_server_member_registration_message(message).await,
        }
    }

    async fn on_server_member_replication_message(&mut self, message: ServerMemberReplicationMessage) {
    }

    async fn on_server_member_registration_message(&mut self, message: ServerMemberRegistrationMessage) {
    }
}
