use crate::client::{
    ClientExitMessageInput, ClientMessage, ClientNotificationEndMessageInput,
    ClientNotificationImmediateMessageInput, ClientNotificationNextMessageInput,
    ClientNotificationStartMessageInput,
};
use crate::common::message::MessageQueueSender;
use crate::common::module::ModuleId;
use crate::common::utility::Bytes;
use futures::Stream;
use futures::StreamExt;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ClientHandle {
    client_message_sender: MessageQueueSender<ClientMessage>,
}

impl ClientHandle {
    pub(crate) fn new(client_message_sender: MessageQueueSender<ClientMessage>) -> Self {
        Self {
            client_message_sender,
        }
    }

    pub async fn exit(&self) {
        self.client_message_sender
            .do_send_asynchronous(ClientExitMessageInput::new())
            .await;
    }

    pub fn server(&self) -> ClientServerHandle {
        ClientServerHandle::new(self.client_message_sender.clone())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ClientServerHandle {
    client_message_sender: MessageQueueSender<ClientMessage>,
}

impl ClientServerHandle {
    pub(crate) fn new(client_message_sender: MessageQueueSender<ClientMessage>) -> Self {
        Self {
            client_message_sender,
        }
    }

    pub async fn notify<T>(&self, module_id: ModuleId, mut body: T)
    where
        T: Stream<Item = Bytes> + Unpin,
    {
        let first = match body.next().await {
            None => return,
            Some(first) => first,
        };

        let mut previous = match body.next().await {
            None => {
                self.client_message_sender
                    .do_send_asynchronous(ClientNotificationImmediateMessageInput::new(
                        module_id, first,
                    ))
                    .await;

                return;
            }
            Some(previous) => previous,
        };

        let (notification_id,) = self
            .client_message_sender
            .do_send_synchronous(ClientNotificationStartMessageInput::new(module_id, first))
            .await
            .into();

        loop {
            let next = match body.next().await {
                None => {
                    self.client_message_sender
                        .do_send_asynchronous(ClientNotificationEndMessageInput::new(
                            notification_id,
                            previous,
                        ))
                        .await;

                    break;
                }
                Some(next) => next,
            };

            self.client_message_sender
                .do_send_asynchronous(ClientNotificationNextMessageInput::new(
                    notification_id,
                    previous,
                ))
                .await;

            previous = next;
        }
    }
}
