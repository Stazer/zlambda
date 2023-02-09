use crate::client::{ClientMessage, ClientNotifyMessageInput};
use crate::common::message::MessageQueueSender;
use crate::common::module::ModuleId;
use crate::common::utility::Bytes;

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

    pub async fn notify(&self, module_id: ModuleId, body: Bytes) {
        self.client_message_sender
            .do_send_asynchronous(ClientNotifyMessageInput::new(module_id, body))
            .await;
    }
}
