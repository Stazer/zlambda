use crate::client::{ClientMessage, ClientNotificationStartMessageInput, ClientNotificationNextMessageInput};
use crate::common::message::MessageQueueSender;
use crate::common::module::ModuleId;
use crate::common::utility::Bytes;
use futures::StreamExt;
use futures::Stream;

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

    pub async fn notify<T>(&self, module_id: ModuleId, mut body: T)
    where
        T: Stream<Item = Bytes> + Unpin,
    {
        let output = self.client_message_sender.do_send_synchronous(
            ClientNotificationStartMessageInput::new(
                module_id,
            ),
        ).await;

        while let Some(bytes) = body.next().await {
            self.client_message_sender.do_send_asynchronous(
                ClientNotificationNextMessageInput::new(
                    output.notification_id(),
                    Some(bytes),
                ),
            ).await;
        }

        self.client_message_sender.do_send_asynchronous(
            ClientNotificationNextMessageInput::new(
                output.notification_id(),
                None,
            ),
        ).await;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ClientServerHandle {
    client_message_sender: MessageQueueSender<ClientMessage>,
}

impl ClientServerHandle {
    pub fn notify(&self, module_id: ModuleId, body: impl Stream<Item = Bytes>) {
    }
}
