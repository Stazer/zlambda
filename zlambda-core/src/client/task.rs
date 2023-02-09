use crate::client::{
    ClientHandle, ClientMessage, ClientModule, ClientNotifyMessage, NewClientError,
    ClientModuleNotifyEventInput,
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
    GeneralMessage, GeneralNotifyMessage, GeneralNotifyMessageInput,
};
use std::sync::Arc;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ClientTask {
    running: bool,
    general_sender: MessageSocketSender<GeneralMessage>,
    general_receiver: MessageSocketReceiver<GeneralMessage>,
    client_message_receiver: MessageQueueReceiver<ClientMessage>,
    client_message_sender: MessageQueueSender<ClientMessage>,
    module_manager: ModuleManager<dyn ClientModule>,
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
        match message {
            ClientMessage::Notify(message) => self.on_client_notify_message(message).await,
        }
    }

    async fn on_client_notify_message(&mut self, message: ClientNotifyMessage) {
        let (input,) = message.into();
        let (module_id, body) = input.into();

        if let Err(error) = self
            .general_sender
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
                todo!()
            },
        }
    }

    async fn on_general_notify_message(&mut self, message: GeneralNotifyMessage) {
        let (input,) = message.into();
        let (module_id, body) = input.into();

        let module = match self.module_manager.get_by_module_id(module_id) {
            None => return,
            Some(module) => module,
        };

        let client_handle = self.handle();
        let module = module.clone();

        spawn(async move {
            module
                .on_notify(ClientModuleNotifyEventInput::new(
                    client_handle,
                    body,
                ))
                .await;
        });
    }
}
