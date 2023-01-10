use crate::follower::connection::FollowerConnectionTask;
use crate::follower::FollowerHandle;
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowerConnectionBuilder {}

impl FollowerConnectionBuilder {
    pub fn task(
        self,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        follower_handle: FollowerHandle,
    ) -> FollowerConnectionTask {
        FollowerConnectionTask::new(reader, writer, follower_handle)
    }
}
