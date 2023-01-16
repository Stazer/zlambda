use crate::message::{AsynchronousMessage, SynchronousMessage};
use crate::server::{
    ServerRegistrationAttemptMessageInput, ServerRegistrationAttemptMessageOutput,
    ServerSocketAcceptMessageInput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerSocketAcceptMessage = AsynchronousMessage<ServerSocketAcceptMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ServerRegistrationAttemptMessage = SynchronousMessage<
    ServerRegistrationAttemptMessageInput,
    ServerRegistrationAttemptMessageOutput,
>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ServerMessage {
    SocketAccept(ServerSocketAcceptMessage),
}

impl From<ServerSocketAcceptMessageInput> for ServerMessage {
    fn from(input: ServerSocketAcceptMessageInput) -> Self {
        Self::SocketAccept(ServerSocketAcceptMessage::new(input))
    }
}
