use crate::leader::follower::LeaderFollowerBuilder;
use crate::leader::follower::LeaderFollowerHandle;
use std::error::Error;
use std::io;
use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::message::{ClientToNodeMessage, MessageError};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderConnectionClientRegistrationResult {
    message: ClientToNodeMessage,
}

impl From<LeaderConnectionClientRegistrationResult> for (ClientToNodeMessage,) {
    fn from(result: LeaderConnectionClientRegistrationResult) -> Self {
        (result.message,)
    }
}

impl LeaderConnectionClientRegistrationResult {
    pub fn new(message: ClientToNodeMessage) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &ClientToNodeMessage {
        &self.message
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderConnectionFollowerRegistrationResult {
    node_id: NodeId,
    builder: LeaderFollowerBuilder,
}

impl From<LeaderConnectionFollowerRegistrationResult> for (NodeId, LeaderFollowerBuilder) {
    fn from(result: LeaderConnectionFollowerRegistrationResult) -> Self {
        (result.node_id, result.builder)
    }
}

impl LeaderConnectionFollowerRegistrationResult {
    pub fn new(node_id: NodeId, builder: LeaderFollowerBuilder) -> Self {
        Self { node_id, builder }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn builder(&self) -> &LeaderFollowerBuilder {
        &self.builder
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderConnectionFollowerRecoveryResult {
    term: Term,
    acknowledging_log_entry_data: Vec<LogEntryData>,
    last_committed_log_entry_id: Option<LogEntryId>,
    follower_handle: LeaderFollowerHandle,
}

impl From<LeaderConnectionFollowerRecoveryResult>
    for (
        Term,
        Vec<LogEntryData>,
        Option<LogEntryId>,
        LeaderFollowerHandle,
    )
{
    fn from(result: LeaderConnectionFollowerRecoveryResult) -> Self {
        (
            result.term,
            result.acknowledging_log_entry_data,
            result.last_committed_log_entry_id,
            result.follower_handle,
        )
    }
}

impl LeaderConnectionFollowerRecoveryResult {
    pub fn new(
        term: Term,
        acknowledging_log_entry_data: Vec<LogEntryData>,
        last_committed_log_entry_id: Option<LogEntryId>,
        follower_handle: LeaderFollowerHandle,
    ) -> Self {
        Self {
            term,
            acknowledging_log_entry_data,
            last_committed_log_entry_id,
            follower_handle,
        }
    }

    pub fn term(&self) -> Term {
        self.term
    }

    pub fn acknowledging_log_entry_data(&self) -> &Vec<LogEntryData> {
        &self.acknowledging_log_entry_data
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn follower_handle(&self) -> &LeaderFollowerHandle {
        &self.follower_handle
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderConnectionResult {
    Stop,
    ConnectionClosed,
    Error(Box<dyn Error + Send>),
    ClientRegistration(LeaderConnectionClientRegistrationResult),
    FollowerRegistration(LeaderConnectionFollowerRegistrationResult),
    FollowerRecovery(LeaderConnectionFollowerRecoveryResult),
}

impl From<Box<dyn Error + Send>> for LeaderConnectionResult {
    fn from(error: Box<dyn Error + Send>) -> Self {
        Self::Error(error)
    }
}

impl From<MessageError> for LeaderConnectionResult {
    fn from(error: MessageError) -> Self {
        Self::Error(Box::new(error))
    }
}

impl From<io::Error> for LeaderConnectionResult {
    fn from(error: io::Error) -> Self {
        if matches!(error.kind(), io::ErrorKind::ConnectionReset) {
            return Self::ConnectionClosed;
        }

        Self::Error(Box::new(error))
    }
}
