use crate::cluster::{
    ConsensusActorBroadcastMessage, ConsensusActorReceiveBeginRequestMessage,
    ConsensusActorReceiveBeginResponseMessage, ConsensusActorReceiveCommitRequestMessage,
    ConsensusActorReceiveCommitResponseMessage, ConsensusActorSendBeginRequestMessage,
    ConsensusActorSendBeginResponseMessage, ConsensusActorSendCommitRequestMessage,
    ConsensusActorSendCommitResponseMessage, ConsensusBeginRequest, ConsensusMessageId,
    ConsensusTransaction, ConsensusTransactionId,
};
use actix::dev::ToEnvelope;
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message,
    ResponseActFuture, WrapFuture,
};
use futures::FutureExt;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::{PhantomData, Unpin};
use tracing::{error, trace};

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
    message_type: PhantomData<C>,
    recipient: Addr<R>,
    _next_message_id: ConsensusMessageId,
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

impl<C, R> Handler<ConsensusActorBroadcastMessage<C>> for ConsensusActor<C, R>
where
    C: Debug + Unpin + Send + 'static,
    R: Actor
        + Debug
        + Handler<ConsensusActorSendBeginRequestMessage<C>>
        + Handler<ConsensusActorSendBeginResponseMessage<C>>
        + Handler<ConsensusActorSendCommitRequestMessage<C>>
        + Handler<ConsensusActorSendCommitResponseMessage<C>>
        + 'static,
    R::Context: ToEnvelope<R, ConsensusActorSendBeginRequestMessage<C>>,
{
    type Result = ResponseActFuture<Self, <ConsensusActorBroadcastMessage<C> as Message>::Result>;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: ConsensusActorBroadcastMessage<C>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (message,): (C,) = message.into();

        let recipient = self.recipient.clone();

        async move {
            recipient
                .send(ConsensusActorSendBeginRequestMessage::<C>::new(
                    ConsensusBeginRequest::new(ConsensusTransactionId::new(0, 0)),
                ))
                .await
        }
        .into_actor(self)
        .map(|result, _actor, context| {
            if let Err(e) = result {
                error!("{}", e);
                context.stop();
            }
        })
        .boxed_local()
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
    R::Context: ToEnvelope<R, ConsensusActorSendBeginRequestMessage<C>>
        + ToEnvelope<R, ConsensusActorSendBeginResponseMessage<C>>
        + ToEnvelope<R, ConsensusActorSendCommitRequestMessage<C>>
        + ToEnvelope<R, ConsensusActorSendCommitResponseMessage<C>>,
{
    pub fn new(recipient: Addr<R>) -> Addr<Self> {
        (Self {
            message_type: PhantomData::default(),
            recipient: recipient,
            _next_message_id: 0,
            _transactions: HashMap::default(),
        })
        .start()
    }
}
