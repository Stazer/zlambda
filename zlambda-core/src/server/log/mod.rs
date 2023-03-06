mod data;
mod entry;
mod error;
mod following;
mod id;
mod leading;
mod manager;
mod term;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use data::*;
pub use entry::*;
pub use error::*;
pub use following::*;
pub use id::*;
pub use leading::*;
pub use manager::*;
pub use term::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogLeaderType = LeadingLog;
pub type LogFollowerType = FollowingLog;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LogType {
    Leader(LeadingLog),
    Follower(FollowingLog),
}

impl From<LogLeaderType> for LogType {
    fn from(r#type: LogLeaderType) -> Self {
        Self::Leader(r#type)
    }
}

impl From<LogFollowerType> for LogType {
    fn from(r#type: LogFollowerType) -> Self {
        Self::Follower(r#type)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Log {
    id: LogId,
    r#type: LogType,
}

impl Log {
    pub fn new(id: LogId, r#type: LogType) -> Self {
        Self { id, r#type }
    }

    pub fn id(&self) -> LogId {
        self.id
    }

    pub fn r#type(&self) -> &LogType {
        &self.r#type
    }

    pub fn type_mut(&mut self) -> &mut LogType {
        &mut self.r#type
    }

    pub fn current_term(&self) -> LogTerm {
        match &self.r#type {
            LogType::Leader(leader) => leader.current_term(),
            LogType::Follower(follower) => follower.current_term(),
        }
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        match &self.r#type {
            LogType::Leader(leader) => leader.last_committed_log_entry_id(),
            LogType::Follower(follower) => follower.last_committed_log_entry_id(),
        }
    }

    pub fn entries(&self) -> LogEntries<'_> {
        LogEntries::new(self)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LogEntries<'a> {
    log: &'a Log,
}

impl<'a> LogEntries<'a> {
    pub(crate) fn new(log: &'a Log) -> Self {
        Self { log }
    }

    pub fn get(&self, id: LogEntryId) -> Option<&'a LogEntry> {
        match self.log.r#type() {
            LogType::Leader(leader) => leader.entries().get(usize::from(id)),
            LogType::Follower(follower) => match follower.entries().get(usize::from(id)) {
                None | Some(None) => None,
                Some(Some(log_entry)) => Some(log_entry),
            },
        }
    }

    /*pub fn iter(&self) -> Box<dyn Iterator<Item = &'a LogEntry> + 'a> {
        match self.log.r#type() {
            LogType::Leader(leader) => Box::new(leader.entries().iter()),
            LogType::Follower(follower) => Box::new(follower.entries().iter().flatten()),
        }
    }*/
}
