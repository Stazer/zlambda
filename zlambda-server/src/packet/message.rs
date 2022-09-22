use actix::{Message};
use zlambda_common::packet::{OperationRequestPacket, ReadPacketError};
use std::marker::PhantomData;
use bytes::Bytes;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ReadPacketMessage<T>
where
    T: OperationRequestPacket,
{
    bytes: Bytes,
    r#type: PhantomData<T>,
}

impl<T> From<ReadPacketMessage<T>> for (Bytes,)
where
    T: OperationRequestPacket,
{
    fn from(packet: ReadPacketMessage<T>) -> Self {
        (packet.bytes,)
    }
}

impl<T> Message for ReadPacketMessage<T>
where
    T: OperationRequestPacket + 'static,
{
    type Result = Result<(), ReadPacketError>;
}

impl<T> ReadPacketMessage<T>
where
    T: OperationRequestPacket + 'static,
{
    pub fn new(bytes: Bytes) -> Self {
        Self {
            bytes,
            r#type: PhantomData::<T>
        }
    }

    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut Bytes {
        &mut self.bytes
    }

    pub fn set_bytes(&mut self, bytes: Bytes) {
        self.bytes = bytes
    }
}
