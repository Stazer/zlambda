use crate::common::utility::TaggedType;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LogIdTag;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogId = TaggedType<usize, LogIdTag>;
