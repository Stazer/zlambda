use std::error::Error;
use std::io;
use zlambda_common::message::{ClientToNodeMessage, MessageError};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerConnectionClientRegistrationResult {
    message: ClientToNodeMessage,
}

impl From<FollowerConnectionClientRegistrationResult> for (ClientToNodeMessage,) {
    fn from(result: FollowerConnectionClientRegistrationResult) -> Self {
        (result.message,)
    }
}

impl FollowerConnectionClientRegistrationResult {
    pub fn new(message: ClientToNodeMessage) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &ClientToNodeMessage {
        &self.message
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerConnectionFollowerHandshakeResult {

}

impl FollowerConnectionFollowerHandshakeResult {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum FollowerConnectionResult {
    Stop,
    ConnectionClosed,
    Error(Box<dyn Error + Send>),
    ClientRegistration(FollowerConnectionClientRegistrationResult),
    //FollowerHandshake(FollowerConnectionFollowerHandshakeResult),
}

impl From<Box<dyn Error + Send>> for FollowerConnectionResult {
    fn from(error: Box<dyn Error + Send>) -> Self {
        Self::Error(error)
    }
}

impl From<MessageError> for FollowerConnectionResult {
    fn from(error: MessageError) -> Self {
        Self::Error(Box::new(error))
    }
}

impl From<io::Error> for FollowerConnectionResult {
    fn from(error: io::Error) -> Self {
        if matches!(error.kind(), io::ErrorKind::ConnectionReset) {
            return Self::ConnectionClosed;
        }

        Self::Error(Box::new(error))
    }
}
