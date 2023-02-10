use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::common::runtime::{select, spawn};
use crate::common::utility::Bytes;
use crate::general::{
    GeneralClientRegistrationResponseMessage, GeneralClientRegistrationResponseMessageInput,
    GeneralMessage, GeneralNotificationMessage, GeneralNotificationMessageInputType,
    GeneralNotifyMessage, GeneralNotifyMessageInput,
};
use crate::server::client::{ServerClientId, ServerClientMessage, ServerClientNotifyMessage};
use crate::server::{
    ServerClientResignationMessageInput, ServerHandle, ServerMessage, ServerModuleGetMessageInput,
    ServerModuleNotificationEventBody, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventInputClientSource, ServerNotifyMessageInput,
    ServerNotifyMessageInputClientSource,
};
use std::collections::HashMap;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerClientTask {
    client_id: ServerClientId,
    server_message_sender: MessageQueueSender<ServerMessage>,
    general_message_sender: MessageSocketSender<GeneralMessage>,
    general_message_receiver: MessageSocketReceiver<GeneralMessage>,
    sender: MessageQueueSender<ServerClientMessage>,
    receiver: MessageQueueReceiver<ServerClientMessage>,
    notification_senders: HashMap<usize, MessageQueueSender<Bytes>>,
}

impl ServerClientTask {
    pub fn new(
        client_id: ServerClientId,
        server_message_sender: MessageQueueSender<ServerMessage>,
        general_message_sender: MessageSocketSender<GeneralMessage>,
        general_message_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            client_id,
            server_message_sender,
            general_message_sender,
            general_message_receiver,
            sender,
            receiver,
            notification_senders: HashMap::default(),
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

        self.server_message_sender
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
            GeneralMessage::Notification(message) => {
                self.on_general_notification_message(message).await
            }
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

        self.server_message_sender
            .do_send_asynchronous(ServerNotifyMessageInput::new(
                module_id,
                ServerNotifyMessageInputClientSource::new(self.client_id).into(),
                body,
            ))
            .await;
    }

    async fn on_general_notification_message(&mut self, message: GeneralNotificationMessage) {
        let (input,) = message.into();
        let (r#type, body) = input.into();

        match r#type {
            GeneralNotificationMessageInputType::Immediate(r#type) => {
                let output = self
                    .server_message_sender
                    .do_send_synchronous(ServerModuleGetMessageInput::new(r#type.module_id()))
                    .await;

                let module = match output.into() {
                    (None,) => return,
                    (Some(module),) => module,
                };

                let handle = ServerHandle::new(self.server_message_sender.clone());
                let client_source =
                    ServerModuleNotificationEventInputClientSource::new(self.client_id);

                let (sender, receiver) = message_queue();
                sender.do_send(body).await;

                spawn(async move {
                    module
                        .on_notification(ServerModuleNotificationEventInput::new(
                            handle,
                            client_source.into(),
                            ServerModuleNotificationEventBody::new(receiver),
                        ))
                        .await;
                });
            }
            GeneralNotificationMessageInputType::Start(r#type) => {
                let output = self
                    .server_message_sender
                    .do_send_synchronous(ServerModuleGetMessageInput::new(r#type.module_id()))
                    .await;

                let module = match output.into() {
                    (None,) => return,
                    (Some(module),) => module,
                };

                let (sender, receiver) = message_queue();
                sender.do_send(body).await;
                self.notification_senders
                    .insert(r#type.notification_id(), sender);

                let handle = ServerHandle::new(self.server_message_sender.clone());
                let client_source =
                    ServerModuleNotificationEventInputClientSource::new(self.client_id);

                spawn(async move {
                    module
                        .on_notification(ServerModuleNotificationEventInput::new(
                            handle,
                            client_source.into(),
                            ServerModuleNotificationEventBody::new(receiver),
                        ))
                        .await;
                });
            }
            GeneralNotificationMessageInputType::Next(r#type) => {
                if let Some(sender) = self.notification_senders.get(&r#type.notification_id()) {
                    sender.do_send(body).await;
                }
            }
            GeneralNotificationMessageInputType::End(r#type) => {
                if let Some(sender) = self.notification_senders.remove(&r#type.notification_id()) {
                    sender.do_send(body).await;
                }
            }
        };
    }
}
