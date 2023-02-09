use crate::client::ClientNotifyMessageInput;
use crate::common::message::AsynchronousMessage;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientNotifyMessage = AsynchronousMessage<ClientNotifyMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClientMessage {
    Notify(ClientNotifyMessage),
}

impl From<ClientNotifyMessage> for ClientMessage {
    fn from(message: ClientNotifyMessage) -> Self {
        Self::Notify(message)
    }
}
