////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum NodeMessage {
    FollowerRegistrationAttempt,
    FollowerRegistrationAcknowledgement,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct NodeMessageChannel
