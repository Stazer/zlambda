use crate::client::{
    ClientExitMessage, ClientHandle, ClientMessage, ClientModule, ClientModuleFinalizeEventInput,
    ClientModuleInitializeEventInput, ClientModuleNotificationEventInput,
    ClientModuleNotificationEventInputBody, ClientNotificationEndMessage,
    ClientNotificationImmediateMessage, ClientNotificationNextMessage,
    ClientNotificationStartMessage, ClientNotificationStartMessageOutput, NewClientError,
};
use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::common::module::ModuleManager;
use crate::common::net::{TcpStream, ToSocketAddrs};
use crate::common::runtime::{select, spawn};
use crate::common::utility::Bytes;
use crate::general::{
    GeneralClientRegistrationRequestMessage, GeneralClientRegistrationRequestMessageInput,
    GeneralMessage, GeneralNotificationMessage, GeneralNotificationMessageInput,
    GeneralNotificationMessageInputEndType, GeneralNotificationMessageInputImmediateType,
    GeneralNotificationMessageInputNextType, GeneralNotificationMessageInputStartType,
    GeneralNotificationMessageInputType,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ClientTask {
    running: bool,
    general_message_sender: MessageSocketSender<GeneralMessage>,
    general_message_receiver: MessageSocketReceiver<GeneralMessage>,
    client_message_receiver: MessageQueueReceiver<ClientMessage>,
    client_message_sender: MessageQueueSender<ClientMessage>,
    module_manager: ModuleManager<dyn ClientModule>,
    incoming_notification_senders: HashMap<usize, MessageQueueSender<Bytes>>,
    outgoing_notification_counter: usize,
}

impl ClientTask {
    pub async fn new<T>(
        address: T,
        modules: impl Iterator<Item = Box<dyn ClientModule>>,
    ) -> Result<Self, NewClientError>
    where
        T: ToSocketAddrs,
    {
        let mut module_manager = ModuleManager::default();

        for module in modules {
            module_manager.load(Arc::from(module))?;
        }

        let socket = TcpStream::connect(address).await?;

        let (reader, writer) = socket.into_split();

        let (mut general_message_sender, mut general_message_receiver) = (
            MessageSocketSender::<GeneralMessage>::new(writer),
            MessageSocketReceiver::<GeneralMessage>::new(reader),
        );

        general_message_sender
            .send(GeneralClientRegistrationRequestMessage::new(
                GeneralClientRegistrationRequestMessageInput,
            ))
            .await?;

        match general_message_receiver.receive().await? {
            None => return Err(MessageError::ExpectedMessage.into()),
            Some(GeneralMessage::ClientRegistrationResponse(_)) => {}
            Some(message) => {
                return Err(MessageError::UnexpectedMessage(format!("{message:?}")).into())
            }
        }

        let (client_message_sender, client_message_receiver) = message_queue();

        Ok(Self {
            running: true,
            general_message_sender,
            general_message_receiver,
            client_message_sender,
            client_message_receiver,
            module_manager,
            incoming_notification_senders: HashMap::default(),
            outgoing_notification_counter: 0,
        })
    }

    pub fn handle(&self) -> ClientHandle {
        ClientHandle::new(self.client_message_sender.clone())
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        for (_module_id, module) in self.module_manager.iter() {
            let client_handle = self.handle();
            let module = module.clone();

            spawn(async move {
                module
                    .on_initialize(ClientModuleInitializeEventInput::new(client_handle))
                    .await;
            });
        }

        while self.running {
            self.select().await
        }

        for (_module_id, module) in self.module_manager.iter() {
            let client_handle = self.handle();
            let module = module.clone();

            spawn(async move {
                module
                    .on_finalize(ClientModuleFinalizeEventInput::new(client_handle))
                    .await;
            });
        }
    }

    async fn select(&mut self) {
        select!(
            message = self.client_message_receiver.do_receive() => {
                self.on_client_message(message).await
            }
            result = self.general_message_receiver.receive() => {
                let message = match result {
                    Ok(Some(message)) => message,
                    Ok(None) => {
                        self.running = false;
                        return;
                    },
                    Err(error) => {
                        error!("{}", error);
                        self.running = false;
                        return;
                    }
                };

                self.on_general_message(message).await
            }
        )
    }

    async fn on_client_message(&mut self, message: ClientMessage) {
        match message {
            ClientMessage::NotificationImmediate(message) => {
                self.on_client_notification_immediate_message(message).await
            }
            ClientMessage::NotificationStart(message) => {
                self.on_client_notification_start_message(message).await
            }
            ClientMessage::NotificationNext(message) => {
                self.on_client_notification_next_message(message).await
            }
            ClientMessage::NotificationEnd(message) => {
                self.on_client_notification_end_message(message).await
            }
            ClientMessage::Exit(message) => self.on_client_exit(message).await,
        }
    }

    async fn on_client_notification_immediate_message(
        &mut self,
        message: ClientNotificationImmediateMessage,
    ) {
        let (input,) = message.into();
        let (module_id, body) = input.into();

        if let Err(error) = self
            .general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputImmediateType::new(module_id, None, None).into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_client_notification_start_message(
        &mut self,
        message: ClientNotificationStartMessage,
    ) {
        let (input, sender) = message.into();
        let (module_id, body) = input.into();

        let notification_id = self.outgoing_notification_counter;
        self.outgoing_notification_counter += 1;

        sender
            .do_send(ClientNotificationStartMessageOutput::new(notification_id))
            .await;

        if let Err(error) = self
            .general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputStartType::new(module_id, notification_id, None, None)
                        .into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_client_notification_next_message(
        &mut self,
        message: ClientNotificationNextMessage,
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

    async fn on_client_notification_end_message(&mut self, message: ClientNotificationEndMessage) {
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

    async fn on_client_exit(&mut self, _message: ClientExitMessage) {
        self.running = false
    }

    async fn on_general_message(&mut self, message: GeneralMessage) {
        match message {
            GeneralMessage::Notification(message) => {
                self.on_general_notification_message(message).await
            }
            _message => {
                todo!()
            }
        }
    }

    async fn on_general_notification_message(&mut self, message: GeneralNotificationMessage) {
        let (input,) = message.into();
        let (r#type, body) = input.into();

        match r#type {
            GeneralNotificationMessageInputType::Immediate(r#type) => {
                let module = match self.module_manager.get_by_module_id(r#type.module_id()) {
                    None => return,
                    Some(module) => module.clone(),
                };

                let handle = self.handle();

                let (sender, receiver) = message_queue();
                sender.do_send(body).await;

                spawn(async move {
                    module
                        .on_notification(ClientModuleNotificationEventInput::new(
                            handle,
                            ClientModuleNotificationEventInputBody::new(receiver),
                        ))
                        .await;
                });
            }
            GeneralNotificationMessageInputType::Start(r#type) => {
                let module = match self.module_manager.get_by_module_id(r#type.module_id()) {
                    None => return,
                    Some(module) => module.clone(),
                };

                let handle = self.handle();

                let (sender, receiver) = message_queue();
                sender.do_send(body).await;
                self.incoming_notification_senders
                    .insert(r#type.notification_id(), sender);

                spawn(async move {
                    module
                        .on_notification(ClientModuleNotificationEventInput::new(
                            handle,
                            ClientModuleNotificationEventInputBody::new(receiver),
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
