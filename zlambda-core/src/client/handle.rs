use crate::client::{ClientMessage};
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
}
