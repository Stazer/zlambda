use crate::packet::ReadPacketMessage;
use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use std::marker::PhantomData;
use zlambda_common::operation::OperationRequest;
use zlambda_common::packet::{from_bytes, OperationRequestPacket};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct PacketReaderActor {}

impl Actor for PacketReaderActor {
    type Context = SyncContext<Self>;
}

impl<T> Handler<ReadPacketMessage<T>> for PacketReaderActor
where
    T: OperationRequestPacket + 'static,
{
    type Result = <ReadPacketMessage<T> as Message>::Result;

    fn handle(
        &mut self,
        message: ReadPacketMessage<T>,
        _: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (bytes,) = message.into();
        let request: OperationRequest = from_bytes::<T>(&bytes)?.into();

        Ok(())
    }
}

impl PacketReaderActor {
    pub fn new() -> Addr<Self> {
        SyncArbiter::start(0, || PacketReaderActor {})
    }
}
