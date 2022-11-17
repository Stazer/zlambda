use crate::node::message::{MessageStreamReader, MessageStreamWriter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerNode {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
}

impl FollowerNode {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
    ) -> Self {
        Self {
            reader,
            writer,
        }
    }

    pub fn reader_mut(&mut self) -> &mut MessageStreamReader {
        &mut self.reader
    }

    pub fn writer_mut(&mut self) -> &mut MessageStreamWriter {
        &mut self.writer
    }
}
