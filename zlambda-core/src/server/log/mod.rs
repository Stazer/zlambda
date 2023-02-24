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

pub mod experimental {
    use crate::server::ServerId;
    use std::any::Any;
    use std::collections::hash_map::RandomState;
    use std::collections::hash_set::Difference;
    use std::collections::HashSet;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub type LogEntryId = crate::server::log::LogEntryId;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub type LogTerm = usize;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub struct LogEntry<T> {
        id: LogEntryId,
        data: T,
    }

    impl<T> LogEntry<T> {
        pub(crate) fn new(id: LogEntryId, data: T) -> Self {
            Self { id, data }
        }

        pub fn id(&self) -> LogEntryId {
            self.id
        }

        pub fn data(&self) -> &T {
            &self.data
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Default)]
    pub struct LogLeadingType<T> {
        current_term: LogTerm,
        entries: Vec<LogEntry<T>>,
        acknowledgeable_server_ids: Vec<HashSet<ServerId>>,
        acknowledged_server_ids: Vec<HashSet<ServerId>>,
        next_committing_log_entry_id: LogEntryId,
    }

    impl<T> LogLeadingType<T> {
        pub(crate) fn new(
            current_term: LogTerm,
            entries: Vec<LogEntry<T>>,
            acknowledgeable_server_ids: Vec<HashSet<ServerId>>,
            acknowledged_server_ids: Vec<HashSet<ServerId>>,
            next_committing_log_entry_id: LogEntryId,
        ) -> Self {
            Self {
                current_term,
                entries,
                acknowledgeable_server_ids,
                acknowledged_server_ids,
                next_committing_log_entry_id,
            }
        }

        pub fn current_term(&self) -> LogTerm {
            self.current_term
        }

        pub fn entries(&self) -> &Vec<LogEntry<T>> {
            &self.entries
        }

        pub fn acknowledgeable_server_ids(&self) -> &Vec<HashSet<ServerId>> {
            &self.acknowledgeable_server_ids
        }

        pub fn acknowledged_server_ids(&self) -> &Vec<HashSet<ServerId>> {
            &self.acknowledged_server_ids
        }

        pub fn next_committing_log_entry_id(&self) -> LogEntryId {
            self.next_committing_log_entry_id
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Default)]
    pub struct LogFollowingType<T> {
        current_term: LogTerm,
        entries: Vec<Option<LogEntry<T>>>,
        next_committing_log_entry_id: LogEntryId,
    }

    impl<T> LogFollowingType<T> {
        pub(crate) fn new(
            current_term: LogTerm,
            entries: Vec<Option<LogEntry<T>>>,
            next_committing_log_entry_id: LogEntryId,
        ) -> Self {
            Self {
                current_term,
                entries,
                next_committing_log_entry_id,
            }
        }

        pub fn current_term(&self) -> LogTerm {
            self.current_term
        }

        pub fn entries(&self) -> &Vec<Option<LogEntry<T>>> {
            &self.entries
        }

        pub fn next_committing_log_entry_id(&self) -> LogEntryId {
            self.next_committing_log_entry_id
        }
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

        pub fn next_committing_log_entry_id(&self) -> LogEntryId {
            match &self.r#type {
                LogType::Leading(leading) => leading.next_committing_log_entry_id(),
                LogType::Following(following) => following.next_committing_log_entry_id(),
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Default)]
    pub struct LogManager {
        logs: Vec<Option<Box<dyn Any>>>,
    }
}
