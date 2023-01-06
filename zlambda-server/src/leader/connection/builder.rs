use crate::leader::connection::LeaderConnectionTask;
use crate::leader::LeaderHandle;
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LeaderConnectionBuilder {}

impl LeaderConnectionBuilder {
    pub fn task(
        self,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_handle: LeaderHandle,
    ) -> LeaderConnectionTask {
        LeaderConnectionTask::new(reader, writer, leader_handle)
    }
}
