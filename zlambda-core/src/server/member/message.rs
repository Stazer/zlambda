use crate::message::AsynchronousMessage;
use crate::server::member::ServerMemberReplicateMessageInput;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerMemberReplicateMessage = AsynchronousMessage<ServerMemberReplicateMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerMemberMessage {
    Replicate(ServerMemberReplicateMessage),
}

impl From<ServerMemberReplicateMessage> for ServerMemberMessage {
    fn from(message: ServerMemberReplicateMessage) -> Self {
        Self::Replicate(message)
    }
}
