mod data;
mod entry;
mod error;
mod following;
mod leading;
mod term;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use data::*;
pub use entry::*;
pub use error::*;
pub use following::*;
pub use leading::*;
pub use term::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

/*pub mod experimental {
    use crate::server::ServerId;
    use std::any::Any;
    use std::collections::hash_map::RandomState;
    use std::collections::hash_set::Difference;
    use std::collections::HashSet;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub struct LogEntryId {

    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub type LogTerm = usize;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub struct LogEntry<T> {
        id: LogEntryId,
        data: T,
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Default)]
    pub struct LogLeadingType<T> {
        term: LogTerm,
        entries: Vec<LogEntry<T>>,
        next_committing_log_entry_id: LogEntryId,
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Default)]
    pub struct LogFollowingType<T> {
        term: LogTerm,
        entries: Vec<Option<LogEntry<T>>>,
        acknowledgeable_server_ids: Vec<HashSet<ServerId>>,
        acknowledged_server_ids: Vec<HashSet<ServerId>>,
        next_committing_log_entry_id: LogEntryId,
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub enum LogType<T> {
        Leading(LogLeadingType<T>),
        Following(LogFollowingType<T>),
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub struct Log<T> {
        r#type: LogType<T>,
    }

    impl<T> Log<T> {
        pub fn r#type(&self) -> &LogType<T> {
            &self.r#type
        }

        pub fn type_mut(&mut self) -> &mut LogType<T> {
            &mut self.r#type
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Default)]
    pub struct LogManager {
        logs: Vec<Option<Box<dyn Any>>>,
    }
}*/
