use crate::log::LogEntryType;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct State {}

impl State {
    pub fn apply(&mut self, log_entry_type: LogEntryType) {}
}
