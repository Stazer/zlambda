use crate::cluster::{
    ConsensusActorExecuteMessage, ConsensusActorReceiveBeginRequestMessage,
    ConsensusActorReceiveBeginResponseMessage, ConsensusActorReceiveCommitRequestMessage,
    ConsensusActorReceiveCommitResponseMessage, ConsensusActorSendBeginRequestMessage,
    ConsensusActorSendBeginResponseMessage, ConsensusActorSendCommitRequestMessage,
    ConsensusActorSendCommitResponseMessage, ConsensusBeginRequest, ConsensusBeginResponse,
    ConsensusCommandId, ConsensusCommitRequest, ConsensusCommitResponse, ConsensusTransaction,
    ConsensusTransactionId,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::{PhantomData, Unpin};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActor<C, R>
where
    C: Debug + Unpin + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
{
    _command_type: PhantomData<C>,
    _recipient: Addr<R>,
    _next_command_id: ConsensusCommandId,
    _transactions: HashMap<ConsensusTransactionId, ConsensusTransaction<C>>,
}

impl<C, R> Actor for ConsensusActor<C, R>
where
    C: Debug + Unpin + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
{
    type Context = Context<Self>;
}

impl<C, R> Handler<ConsensusActorExecuteMessage<C>> for ConsensusActor<C, R>
where
    C: Debug + Unpin + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
{
    type Result = ();

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: ConsensusActorExecuteMessage<C>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl<C, R> Handler<ConsensusActorReceiveBeginRequestMessage<C>> for ConsensusActor<C, R>
where
    C: Debug + Unpin + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
{
    type Result = ();

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorReceiveBeginRequestMessage<C>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl<C, R> Handler<ConsensusActorReceiveBeginResponseMessage<C>> for ConsensusActor<C, R>
where
    C: Debug + Unpin + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
{
    type Result = ();

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorReceiveBeginResponseMessage<C>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl<C, R> Handler<ConsensusActorReceiveCommitRequestMessage<C>> for ConsensusActor<C, R>
where
    C: Debug + Unpin + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
{
    type Result = ();

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorReceiveCommitRequestMessage<C>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl<C, R> Handler<ConsensusActorReceiveCommitResponseMessage<C>> for ConsensusActor<C, R>
where
    C: Debug + Unpin + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
{
    type Result = ();

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorReceiveCommitResponseMessage<C>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl<C, R> ConsensusActor<C, R>
where
    C: Debug + Unpin + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
{
    pub fn new(recipient: Addr<R>) -> Addr<Self> {
        (Self {
            _command_type: PhantomData::default(),
            _recipient: recipient,
            _next_command_id: 0,
            _transactions: HashMap::default(),
        })
        .start()
    }
}
