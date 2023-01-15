use crate::message::{SynchronousMessageEnvelope, AsynchronousMessageEnvelope};
use crate::node::{NodeInternalRegistrationAttemptMessageInput, NodeInternalRegistrationAttemptMessageOutput};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeInternalMessage {
    RegistrationAttempt(
        SynchronousMessageEnvelope<
            NodeInternalRegistrationAttemptMessageInput,
            NodeInternalRegistrationAttemptMessageOutput,
       >,
    ),
}
