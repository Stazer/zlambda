use crate::cluster::{
    ConsensusBeginRequest, ConsensusBeginResponse, ConsensusCommitRequest, ConsensusCommitResponse,
};
use actix::Message;
use std::fmt::Debug;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorBroadcastMessage<C>
where
    C: Debug + 'static,
{
    message: C,
}

impl<C> From<ConsensusActorBroadcastMessage<C>> for (C,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorBroadcastMessage<C>) -> Self {
        (message.message,)
    }
}

impl<C> Message for ConsensusActorBroadcastMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorBroadcastMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(message: C) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &C {
        &self.message
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorSendBeginRequestMessage<C>
where
    C: Debug + 'static,
{
    request: ConsensusBeginRequest<C>,
}

impl<C> From<ConsensusActorSendBeginRequestMessage<C>> for (ConsensusBeginRequest<C>,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorSendBeginRequestMessage<C>) -> Self {
        (message.request,)
    }
}

impl<C> Message for ConsensusActorSendBeginRequestMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorSendBeginRequestMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(request: ConsensusBeginRequest<C>) -> Self {
        Self { request }
    }

    pub fn request(&self) -> &ConsensusBeginRequest<C> {
        &self.request
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorReceiveBeginRequestMessage<C>
where
    C: Debug + 'static,
{
    request: ConsensusBeginRequest<C>,
}

impl<C> From<ConsensusActorReceiveBeginRequestMessage<C>> for (ConsensusBeginRequest<C>,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorReceiveBeginRequestMessage<C>) -> Self {
        (message.request,)
    }
}

impl<C> Message for ConsensusActorReceiveBeginRequestMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorReceiveBeginRequestMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(request: ConsensusBeginRequest<C>) -> Self {
        Self { request }
    }

    pub fn request(&self) -> &ConsensusBeginRequest<C> {
        &self.request
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorSendBeginResponseMessage<C>
where
    C: Debug + 'static,
{
    response: ConsensusBeginResponse<C>,
}

impl<C> From<ConsensusActorSendBeginResponseMessage<C>> for (ConsensusBeginResponse<C>,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorSendBeginResponseMessage<C>) -> Self {
        (message.response,)
    }
}

impl<C> Message for ConsensusActorSendBeginResponseMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorSendBeginResponseMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(response: ConsensusBeginResponse<C>) -> Self {
        Self { response }
    }

    pub fn response(&self) -> &ConsensusBeginResponse<C> {
        &self.response
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorReceiveBeginResponseMessage<C>
where
    C: Debug + 'static,
{
    response: ConsensusBeginResponse<C>,
}

impl<C> From<ConsensusActorReceiveBeginResponseMessage<C>> for (ConsensusBeginResponse<C>,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorReceiveBeginResponseMessage<C>) -> Self {
        (message.response,)
    }
}

impl<C> Message for ConsensusActorReceiveBeginResponseMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorReceiveBeginResponseMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(response: ConsensusBeginResponse<C>) -> Self {
        Self { response }
    }

    pub fn response(&self) -> &ConsensusBeginResponse<C> {
        &self.response
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorSendCommitRequestMessage<C>
where
    C: Debug + 'static,
{
    request: ConsensusCommitRequest<C>,
}

impl<C> From<ConsensusActorSendCommitRequestMessage<C>> for (ConsensusCommitRequest<C>,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorSendCommitRequestMessage<C>) -> Self {
        (message.request,)
    }
}

impl<C> Message for ConsensusActorSendCommitRequestMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorSendCommitRequestMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(request: ConsensusCommitRequest<C>) -> Self {
        Self { request }
    }

    pub fn request(&self) -> &ConsensusCommitRequest<C> {
        &self.request
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorReceiveCommitRequestMessage<C>
where
    C: Debug + 'static,
{
    request: ConsensusCommitRequest<C>,
}

impl<C> From<ConsensusActorReceiveCommitRequestMessage<C>> for (ConsensusCommitRequest<C>,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorReceiveCommitRequestMessage<C>) -> Self {
        (message.request,)
    }
}

impl<C> Message for ConsensusActorReceiveCommitRequestMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorReceiveCommitRequestMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(request: ConsensusCommitRequest<C>) -> Self {
        Self { request }
    }

    pub fn request(&self) -> &ConsensusCommitRequest<C> {
        &self.request
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorSendCommitResponseMessage<C>
where
    C: Debug + 'static,
{
    response: ConsensusCommitResponse<C>,
}

impl<C> From<ConsensusActorSendCommitResponseMessage<C>> for (ConsensusCommitResponse<C>,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorSendCommitResponseMessage<C>) -> Self {
        (message.response,)
    }
}

impl<C> Message for ConsensusActorSendCommitResponseMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorSendCommitResponseMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(response: ConsensusCommitResponse<C>) -> Self {
        Self { response }
    }

    pub fn response(&self) -> &ConsensusCommitResponse<C> {
        &self.response
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConsensusActorReceiveCommitResponseMessage<C>
where
    C: Debug + 'static,
{
    response: ConsensusCommitResponse<C>,
}

impl<C> From<ConsensusActorReceiveCommitResponseMessage<C>> for (ConsensusCommitResponse<C>,)
where
    C: Debug + 'static,
{
    fn from(message: ConsensusActorReceiveCommitResponseMessage<C>) -> Self {
        (message.response,)
    }
}

impl<C> Message for ConsensusActorReceiveCommitResponseMessage<C>
where
    C: Debug + 'static,
{
    type Result = ();
}

impl<C> ConsensusActorReceiveCommitResponseMessage<C>
where
    C: Debug + 'static,
{
    pub fn new(response: ConsensusCommitResponse<C>) -> Self {
        Self { response }
    }

    pub fn response(&self) -> &ConsensusCommitResponse<C> {
        &self.response
    }
}
