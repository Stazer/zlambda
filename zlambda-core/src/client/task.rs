use crate::client::{
    ClientHandle, ClientMessage, ClientModule,
    NewClientError, ClientModuleNotificationEventInput, ClientModuleNotificationEventInputBody,
};
use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::common::module::ModuleManager;
use crate::common::net::{TcpStream, ToSocketAddrs};
use crate::common::runtime::{select, spawn};
use crate::general::{
    GeneralClientRegistrationRequestMessage, GeneralClientRegistrationRequestMessageInput,
    GeneralMessage, GeneralNotificationMessage, GeneralNotificationMessageInputType,
};
use std::sync::Arc;
use std::collections::HashMap;
use crate::common::utility::Bytes;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ClientTask {
    running: bool,
    general_sender: MessageSocketSender<GeneralMessage>,
    general_receiver: MessageSocketReceiver<GeneralMessage>,
    client_message_receiver: MessageQueueReceiver<ClientMessage>,
    client_message_sender: MessageQueueSender<ClientMessage>,
    module_manager: ModuleManager<dyn ClientModule>,
    notification_senders: HashMap<usize, MessageQueueSender<Bytes>>,
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

        let (mut general_sender, mut general_receiver) = (
            MessageSocketSender::<GeneralMessage>::new(writer),
            MessageSocketReceiver::<GeneralMessage>::new(reader),
        );

        general_sender
            .send(GeneralClientRegistrationRequestMessage::new(
                GeneralClientRegistrationRequestMessageInput,
            ))
            .await?;

        match general_receiver.receive().await? {
            None => return Err(MessageError::ExpectedMessage.into()),
            Some(GeneralMessage::ClientRegistrationResponse(_)) => {}
            Some(message) => {
                return Err(MessageError::UnexpectedMessage(format!("{message:?}")).into())
            }
        }

        let (client_message_sender, client_message_receiver) = message_queue();

        Ok(Self {
            running: true,
            general_sender,
            general_receiver,
            client_message_sender,
            client_message_receiver,
            module_manager,
            notification_senders: HashMap::default(),
        })
    }

    pub fn handle(&self) -> ClientHandle {
        ClientHandle::new(self.client_message_sender.clone())
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        while self.running {
            self.select().await
        }
    }

    async fn select(&mut self) {
        select!(
            message = self.client_message_receiver.do_receive() => {
                self.on_client_message(message).await
            }
            result = self.general_receiver.receive() => {
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
    }

    async fn on_general_message(&mut self, message: GeneralMessage) {
        match message {
            GeneralMessage::Notification(message) => {
                self.on_general_notification_message(message).await
            }
            message => {
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
                self.notification_senders.insert(r#type.notification_id(), sender);

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
