use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ConsensusInstanceId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ConsensusCommandId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ConsensusTransactionId {
    command_id: ConsensusCommandId,
    instance_id: ConsensusInstanceId,
}

impl ConsensusTransactionId {
    pub fn new(command_id: ConsensusCommandId, instance_id: ConsensusInstanceId) -> Self {
        Self {
            command_id,
            instance_id,
        }
    }

    pub fn command_id(&self) -> ConsensusCommandId {
        self.command_id
    }

    pub fn instance_id(&self) -> ConsensusInstanceId {
        self.instance_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ConsensusTransactionState<C> {
    Proposed { command: C },
    Promised,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusTransaction<C> {
    id: ConsensusTransactionId,
    state: ConsensusTransactionState<C>,
}

impl<C> ConsensusTransaction<C> {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsensusBeginRequest<C> {
    transaction_id: ConsensusTransactionId,
    command_type: PhantomData<C>,
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
            command_type: PhantomData::<C>,
        }
    }

    pub fn transaction_id(&self) -> &ConsensusTransactionId {
        &self.transaction_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsensusBeginResponse<C> {
    transaction_id: ConsensusTransactionId,
    command_type: PhantomData<C>,
}

impl<C> From<ConsensusBeginResponse<C>> for (ConsensusTransactionId,) {
    fn from(request: ConsensusBeginResponse<C>) -> Self {
        (request.transaction_id,)
    }
}

impl<C> ConsensusBeginResponse<C> {
    pub fn new(transaction_id: ConsensusTransactionId) -> Self {
        Self {
            transaction_id,
            command_type: PhantomData::<C>,
        }
    }

    pub fn transaction_id(&self) -> &ConsensusTransactionId {
        &self.transaction_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsensusCommitRequest<C> {
    id: ConsensusCommandId,
    command_type: PhantomData<C>,
}

impl<C> ConsensusCommitRequest<C> {
    pub fn new(id: ConsensusCommandId) -> Self {
        Self {
            id,
            command_type: PhantomData::<C>,
        }
    }

    pub fn id(&self) -> ConsensusCommandId {
        self.id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsensusCommitResponse<C> {
    id: ConsensusCommandId,
    command_type: PhantomData<C>,
}

impl<C> ConsensusCommitResponse<C> {
    pub fn new(id: ConsensusCommandId) -> Self {
        Self {
            id,
            command_type: PhantomData::<C>,
        }
    }

    pub fn id(&self) -> ConsensusCommandId {
        self.id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait ConsensusProposer<C> {
    async fn send_begin_request(&mut self, request: ConsensusBeginRequest<C>);
    async fn send_commit_request(&mut self, request: ConsensusCommitRequest<C>);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait ConsensusAcceptor<C> {
    async fn send_begin_response(&mut self, reponse: ConsensusBeginResponse<C>);
    async fn send_commit_response(&mut self, response: ConsensusCommitResponse<C>);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait ConsensusCommitter<C> {
    async fn commit(&mut self, command: C);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ConsensusManager<C, H>
where
    H: ConsensusProposer<C> + ConsensusAcceptor<C>,
{
    handler: H,
    command_type: PhantomData<C>,
    next_command_id: ConsensusCommandId,
    transactions: HashMap<ConsensusTransactionId, ConsensusTransaction<C>>,
}

impl<C, H> ConsensusManager<C, H>
where
    H: ConsensusProposer<C> + ConsensusAcceptor<C>,
{
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            command_type: PhantomData::<C>,
            next_command_id: 0,
            transactions: HashMap::default(),
        }
    }

    pub async fn execute(&mut self, command: C, instance_id: ConsensusInstanceId) {
        let command_id = self.next_command_id;
        self.next_command_id += 1;

        let transaction = ConsensusTransaction::new(
            ConsensusTransactionId::new(command_id, instance_id),
            ConsensusTransactionState::Proposed { command },
        );

        let transaction_id = transaction.id().clone();

        self.transactions
            .insert(transaction_id.clone(), transaction);

        self.handler
            .send_begin_request(ConsensusBeginRequest::new(transaction_id))
            .await;
    }

    pub async fn receive_begin_request(&mut self, request: ConsensusBeginRequest<C>) {
        let (transaction_id,): (ConsensusTransactionId,) = request.into();

        self.transactions.insert(
            transaction_id.clone(),
            ConsensusTransaction::new(transaction_id, ConsensusTransactionState::Promised),
        );

        //self.handler
            //.send_begin_response(Consensus)
    }

    pub async fn receive_begin_response(&mut self, response: ConsensusBeginResponse<C>) {

    }

    pub async fn receive_commit_request(&mut self, request: ConsensusCommitRequest<C>) {}

    pub async fn receive_commit_response(&mut self, response: ConsensusCommitResponse<C>) {}
}
