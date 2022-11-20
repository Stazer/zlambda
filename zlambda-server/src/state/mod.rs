use tracing::trace;
use zlambda_common::log::ClientLogEntryType;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct State {}

impl State {
    pub fn apply(&mut self, client_log_entry_type: ClientLogEntryType) {
        trace!("Applying {:?}", &client_log_entry_type);
    }
}
