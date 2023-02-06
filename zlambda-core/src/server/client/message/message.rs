use crate::common::message::AsynchronousMessage;
use crate::server::client::ServerClientNotifyMessageInput;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerClientNotifyMessage = AsynchronousMessage<ServerClientNotifyMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum ServerClientMessage {
    Notify(ServerClientNotifyMessage),
}

impl From<ServerClientNotifyMessage> for ServerClientMessage {
    fn from(message: ServerClientNotifyMessage) -> Self {
        Self::Notify(message)
    }
}
