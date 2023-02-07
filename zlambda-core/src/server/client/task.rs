use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::common::runtime::{select, spawn};
use crate::general::{
    GeneralClientRegistrationResponseMessage, GeneralClientRegistrationResponseMessageInput,
    GeneralMessage, GeneralNotifyMessage, GeneralNotifyMessageInput,
};
use crate::server::client::{ServerClientId, ServerClientMessage, ServerClientNotifyMessage};
use crate::server::{
    ServerClientResignationMessageInput, ServerMessage, ServerNotifyMessageInput,
    ServerNotifyMessageInputClientSource,
};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerClientTask {
    client_id: ServerClientId,
    server_queue_sender: MessageQueueSender<ServerMessage>,
    general_message_sender: MessageSocketSender<GeneralMessage>,
    general_message_receiver: MessageSocketReceiver<GeneralMessage>,
    sender: MessageQueueSender<ServerClientMessage>,
    receiver: MessageQueueReceiver<ServerClientMessage>,
}

impl ServerClientTask {
    pub fn new(
        client_id: ServerClientId,
        server_queue_sender: MessageQueueSender<ServerMessage>,
        general_message_sender: MessageSocketSender<GeneralMessage>,
        general_message_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            client_id,
            server_queue_sender,
            general_message_sender,
            general_message_receiver,
            sender,
            receiver,
        }
    }

    pub fn sender(&self) -> &MessageQueueSender<ServerClientMessage> {
        &self.sender
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        if let Err(error) = self
            .general_message_sender
            .send(GeneralClientRegistrationResponseMessage::new(
                GeneralClientRegistrationResponseMessageInput,
            ))
            .await
        {
            error!("{}", error);
            return;
        }

        loop {
            self.select().await
        }

        self.server_queue_sender
            .do_send_asynchronous(ServerClientResignationMessageInput::new(self.client_id))
            .await;
    }

    async fn select(&mut self) {
        select!(
            message = self.receiver.do_receive() => {
                self.on_server_client_message(message).await
            }
            result = self.general_message_receiver.receive() => {
                match result {
                    Err(error) => {
                        error!("{}", error);
                    }
                    Ok(None) => {
                        error!("Connection loss");
                    }
                    Ok(Some(message)) => {
                        self.on_general_message(message).await
                    }
                }
            }
        )
    }

    async fn on_server_client_message(&mut self, message: ServerClientMessage) {
        match message {
            ServerClientMessage::Notify(message) => {
                self.on_server_client_notify_message(message).await
            }
        }
    }

    async fn on_server_client_notify_message(&mut self, message: ServerClientNotifyMessage) {
        let (input,) = message.into();
        let (module_id, body) = input.into();

        if let Err(error) = self
            .general_message_sender
            .send(GeneralNotifyMessage::new(GeneralNotifyMessageInput::new(
                module_id, body,
            )))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_general_message(&mut self, message: GeneralMessage) {
        match message {
            GeneralMessage::Notify(message) => self.on_general_notify_message(message).await,
            message => {
                error!(
                    "{}",
                    MessageError::UnexpectedMessage(format!("{message:?}"))
                );
            }
        }
    }

    async fn on_general_notify_message(&mut self, message: GeneralNotifyMessage) {
        let (input,) = message.into();
        let (module_id, body) = input.into();

        self.server_queue_sender
            .do_send_asynchronous(ServerNotifyMessageInput::new(
                module_id,
                ServerNotifyMessageInputClientSource::new(self.client_id).into(),
                body,
            ))
            .await;
    }
}
