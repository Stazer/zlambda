mod follower_registration;
mod ping;
mod socket_accept;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use follower_registration::*;
pub use ping::*;
pub use socket_accept::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeMessage {
    Ping(NodePingMessage),
    SocketAccept(NodeSocketAcceptMessage),
    FollowerRegistrationAttempt(NodeFollowerRegistrationAttemptMessage),
    FollowerRegistrationAcknowledgement(NodeFollowerRegistrationAcknowledgementMessage),
}

impl From<NodePingMessage> for NodeMessage {
    fn from(message: NodePingMessage) -> Self {
        Self::Ping(message)
    }
}

impl From<NodeSocketAcceptMessage> for NodeMessage {
    fn from(message: NodeSocketAcceptMessage) -> Self {
        Self::SocketAccept(message)
    }
}

impl From<NodeFollowerRegistrationAttemptMessage> for NodeMessage {
    fn from(message: NodeFollowerRegistrationAttemptMessage) -> Self {
        Self::FollowerRegistrationAttempt(message)
    }
}

impl From<NodeFollowerRegistrationAcknowledgementMessage> for NodeMessage {
    fn from(message: NodeFollowerRegistrationAcknowledgementMessage) -> Self {
        Self::FollowerRegistrationAcknowledgement(message)
    }
}
