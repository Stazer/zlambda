use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;

pub type ConsensusInstanceId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ConsensusMessageId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ConsensusTransactionId {
    message_id: ConsensusMessageId,
    instance_id: ConsensusInstanceId,
}

impl ConsensusTransactionId {
    pub fn new(message_id: ConsensusMessageId, instance_id: ConsensusInstanceId) -> Self {
        Self {
            message_id,
            instance_id,
        }
    }

    pub fn message_id(&self) -> ConsensusMessageId {
        self.message_id
    }

    pub fn instance_id(&self) -> ConsensusInstanceId {
        self.instance_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum ConsensusTransactionState<C>
where
    C: Debug + 'static,
{
    Proposed { message: C },
    Promised,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ConsensusTransaction<C>
where
    C: Debug + 'static,
{
    id: ConsensusTransactionId,
    state: ConsensusTransactionState<C>,
}

impl<C> ConsensusTransaction<C>
where
    C: Debug + 'static,
{
    pub fn new(id: ConsensusTransactionId, state: ConsensusTransactionState<C>) -> Self {
        Self { id, state }
    }

    pub fn id(&self) -> &ConsensusTransactionId {
        &self.id
    }

    pub fn state(&self) -> &ConsensusTransactionState<C> {
        &self.state
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsensusBeginRequest<C> {
    transaction_id: ConsensusTransactionId,
    message_type: PhantomData<C>,
}

impl<C> From<ConsensusBeginRequest<C>> for (ConsensusTransactionId,) {
    fn from(request: ConsensusBeginRequest<C>) -> Self {
        (request.transaction_id,)
    }
}

impl<C> ConsensusBeginRequest<C> {
    pub fn new(transaction_id: ConsensusTransactionId) -> Self {
        Self {
            transaction_id,
            message_type: PhantomData::<C>,
        }
    }

    pub fn transaction_id(&self) -> &ConsensusTransactionId {
        &self.transaction_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConsensusBeginResponse<C> {
    Success {
        transaction_id: ConsensusTransactionId,
        message_type: PhantomData<C>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsensusCommitRequest<C> {
    id: ConsensusMessageId,
    message_type: PhantomData<C>,
}

impl<C> ConsensusCommitRequest<C> {
    pub fn new(id: ConsensusMessageId) -> Self {
        Self {
            id,
            message_type: PhantomData::<C>,
        }
    }

    pub fn id(&self) -> ConsensusMessageId {
        self.id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsensusCommitResponse<C> {
    id: ConsensusMessageId,
    message_type: PhantomData<C>,
}

impl<C> ConsensusCommitResponse<C> {
    pub fn new(id: ConsensusMessageId) -> Self {
        Self {
            id,
            message_type: PhantomData::<C>,
        }
    }

    pub fn id(&self) -> ConsensusMessageId {
        self.id
    }
}
