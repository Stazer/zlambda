use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::common::runtime::{select, spawn};
use crate::common::utility::Bytes;
use crate::general::{
    GeneralClientRegistrationResponseMessage, GeneralClientRegistrationResponseMessageInput,
    GeneralMessage, GeneralNotificationMessage, GeneralNotificationMessageInput,
    GeneralNotificationMessageInputEndType, GeneralNotificationMessageInputImmediateType,
    GeneralNotificationMessageInputNextType, GeneralNotificationMessageInputStartType,
    GeneralNotificationMessageInputType,
};
use crate::server::client::{
    ServerClientId, ServerClientMessage, ServerClientNotificationEndMessage,
    ServerClientNotificationImmediateMessage, ServerClientNotificationNextMessage,
    ServerClientNotificationStartMessage, ServerClientNotificationStartMessageOutput,
    ServerClientShutdownMessage,
};
use crate::server::{
    ServerClientResignationMessageInput, ServerHandle, ServerMessage, ServerModuleGetMessageInput,
    ServerModuleNotificationEventBody, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventInputClientSource
};
use std::collections::HashMap;
use tracing::{error, debug};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerClientTask {
    client_id: ServerClientId,
    running: bool,
    server_message_sender: MessageQueueSender<ServerMessage>,
    general_message_sender: MessageSocketSender<GeneralMessage>,
    general_message_receiver: MessageSocketReceiver<GeneralMessage>,
    sender: MessageQueueSender<ServerClientMessage>,
    receiver: MessageQueueReceiver<ServerClientMessage>,
    incoming_notification_senders: HashMap<usize, MessageQueueSender<Bytes>>,
    outgoing_notification_counter: usize,
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
            running: true,
            server_message_sender,
            general_message_sender,
            general_message_receiver,
            sender,
            receiver,
            incoming_notification_senders: HashMap::default(),
            outgoing_notification_counter: 0,
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

        while self.running {
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
                        self.running = false;
                    }
                    Ok(None) => {
                        debug!("Connection loss");
                        self.running = false;
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
            ServerClientMessage::Shutdown(message) => {
                self.on_server_client_shutdown_message(message).await
            }
            ServerClientMessage::NotificationImmediate(message) => {
                self.on_server_client_notification_immediate_message(message)
                    .await
            }
            ServerClientMessage::NotificationStart(message) => {
                self.on_server_client_notification_start_message(message)
                    .await
            }
            ServerClientMessage::NotificationNext(message) => {
                self.on_server_client_notification_next_message(message)
                    .await
            }
            ServerClientMessage::NotificationEnd(message) => {
                self.on_server_client_notification_end_message(message)
                    .await
            }
        }
    }

    async fn on_server_client_shutdown_message(&mut self, _message: ServerClientShutdownMessage) {
        self.running = false;
    }

    async fn on_server_client_notification_immediate_message(
        &mut self,
        message: ServerClientNotificationImmediateMessage,
    ) {
        let (input,) = message.into();
        let (module_id, body) = input.into();

        if let Err(error) = self
            .general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputImmediateType::new(module_id).into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_server_client_notification_start_message(
        &mut self,
        message: ServerClientNotificationStartMessage,
    ) {
        let (input, sender) = message.into();
        let (module_id, body) = input.into();

        let notification_id = self.outgoing_notification_counter;
        self.outgoing_notification_counter += 1;

        sender
            .do_send(ServerClientNotificationStartMessageOutput::new(
                notification_id,
            ))
            .await;

        if let Err(error) = self
            .general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputStartType::new(module_id, notification_id)
                        .into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_server_client_notification_next_message(
        &mut self,
        message: ServerClientNotificationNextMessage,
    ) {
        let (input,) = message.into();
        let (notification_id, body) = input.into();

        if let Err(error) = self
            .general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputNextType::new(notification_id).into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_server_client_notification_end_message(
        &mut self,
        message: ServerClientNotificationEndMessage,
    ) {
        let (input,) = message.into();
        let (notification_id, body) = input.into();

        if let Err(error) = self
            .general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputEndType::new(notification_id).into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_general_message(&mut self, message: GeneralMessage) {
        match message {
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
                self.incoming_notification_senders
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
                if let Some(sender) = self
                    .incoming_notification_senders
                    .get(&r#type.notification_id())
                {
                    sender.do_send(body).await;
                }
            }
            GeneralNotificationMessageInputType::End(r#type) => {
                if let Some(sender) = self
                    .incoming_notification_senders
                    .remove(&r#type.notification_id())
                {
                    sender.do_send(body).await;
                }
            }
        };
    }
}
