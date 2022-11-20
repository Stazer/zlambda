use crate::log::ClientLogEntryType;
use tracing::trace;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct State {}

impl State {
    pub fn apply(&mut self, log_entry_type: ClientLogEntryType) {
        trace!("Applying {:?}", &log_entry_type);
    }
}
