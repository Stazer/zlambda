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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LogEntries<'a> {
    log: &'a Log,
}

impl<'a> LogEntries<'a> {
    pub fn new(
        log: &'a Log,
    ) -> Self  {
        Self {
            log,
        }
    }

    pub fn get(&'a self, id: LogEntryId) -> Option<&'a LogEntry> {
        match self.log.r#type() {
            LogType::Leader(leader) => leader.entries().get(usize::from(id)),
            LogType::Follower(follower) => match follower.entries().get(usize::from(id)) {
                None | Some(None) => None,
                Some(Some(log_entry)) => Some(log_entry),
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*pub mod experimental {
    use crate::common::sync::RwLock;
    use crate::common::utility::TaggedType;
    use crate::server::log::LogError;
    use crate::server::ServerId;
    use serde::{Deserialize, Serialize};
    use std::any::Any;
    use std::collections::hash_map::RandomState;
    use std::collections::hash_set::Difference;
    use std::collections::HashSet;
    use std::sync::Arc;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub struct LogEntryIdTag;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub type LogEntryId = TaggedType<usize, LogEntryIdTag>;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub struct LogTermTag;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub type LogTerm = TaggedType<usize, LogTermTag>;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Debug, Deserialize, Serialize)]
    pub struct LogEntry<T> {
        id: LogEntryId,
        term: LogTerm,
        data: T,
    }

    impl<T> LogEntry<T> {
        pub(crate) fn new(id: LogEntryId, term: LogTerm, data: T) -> Self {
            Self { id, term, data }
        }

        pub fn id(&self) -> LogEntryId {
            self.id
        }

        pub fn term(&self) -> LogTerm {
            self.term
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

        pub fn quorum_count(&self, id: LogEntryId) -> Option<usize> {
            self.acknowledgeable_server_ids
                .get(usize::from(id))
                .map(|server_ids| server_ids.len() / 2)
        }

        pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
            match usize::from(self.next_committing_log_entry_id) {
                0 => None,
                next_committing_log_entry_id => {
                    Some(LogEntryId::from(next_committing_log_entry_id - 1))
                }
            }
        }

        pub fn remaining_acknowledgeable_server_ids(
            &self,
            id: LogEntryId,
        ) -> Option<Difference<'_, ServerId, RandomState>> {
            match (
                self.acknowledgeable_server_ids.get(usize::from(id)),
                self.acknowledged_server_ids.get(usize::from(id)),
            ) {
                (Some(acknowledgeable_server_ids), Some(acknowledged_server_ids)) => {
                    Some(acknowledgeable_server_ids.difference(acknowledged_server_ids))
                }
                _ => None,
            }
        }

        pub fn acknowledgeable_log_entries_for_server(
            &self,
            server_id: ServerId,
        ) -> impl Iterator<Item = &LogEntry<T>> {
            self.entries
                .iter()
                .skip(usize::from(
                    self.last_committed_log_entry_id().unwrap_or_default(),
                ))
                .filter(move |log_entry| {
                    self.remaining_acknowledgeable_server_ids(log_entry.id())
                        .map(|remaining_acknowledgeable_server_ids| {
                            remaining_acknowledgeable_server_ids
                                .filter(|remaining_server_id| **remaining_server_id == server_id)
                                .next()
                                .is_some()
                        })
                        .unwrap_or(false)
                })
        }

        pub fn is_acknowledged(&self, id: LogEntryId) -> bool {
            match (
                self.quorum_count(id),
                self.remaining_acknowledgeable_server_ids(id),
            ) {
                (Some(quorum_count), Some(remaining_acknowledgeable_server_ids)) => {
                    remaining_acknowledgeable_server_ids.count() <= quorum_count
                }
                _ => false,
            }
        }

        pub fn is_committed(&self, id: LogEntryId) -> bool {
            match self.last_committed_log_entry_id() {
                None => false,
                Some(last_committed_log_entry_id) => last_committed_log_entry_id <= id,
            }
        }

        pub fn acknowledge(&mut self, id: LogEntryId, server_id: ServerId) -> Result<(), LogError> {
            let acknowledgeable_server_ids =
                match self.acknowledgeable_server_ids.get(usize::from(id)) {
                    Some(server_ids) => server_ids,
                    None => return Err(LogError::NotExisting),
                };

            if !acknowledgeable_server_ids.contains(&server_id) {
                return Err(LogError::NotAcknowledgeable);
            }

            let acknowledged_server_ids =
                match self.acknowledged_server_ids.get_mut(usize::from(id)) {
                    Some(server_ids) => server_ids,
                    None => return Err(LogError::NotExisting),
                };

            if acknowledged_server_ids.contains(&server_id) {
                return Err(LogError::AlreadyAcknowledged);
            }

            acknowledged_server_ids.insert(server_id);

            let mut current_log_entry_id = id;

            loop {
                if self.is_acknowledged(current_log_entry_id)
                    && self.next_committing_log_entry_id == current_log_entry_id
                {
                    self.next_committing_log_entry_id =
                        LogEntryId::from(usize::from(self.next_committing_log_entry_id) + 1);
                    current_log_entry_id = LogEntryId::from(usize::from(current_log_entry_id) + 1);
                } else {
                    break;
                }
            }

            Ok(())
        }

        pub fn append(
            &mut self,
            log_entries_data: Vec<T>,
            acknowledgeable_server_ids: HashSet<ServerId>,
        ) -> Vec<LogEntryId> {
            self.entries.reserve(log_entries_data.len());
            let start = self.entries.len();

            self.entries.extend(
                log_entries_data
                    .into_iter()
                    .enumerate()
                    .map(|(index, data)| {
                        LogEntry::new(LogEntryId::from(start + index), self.current_term, data)
                    }),
            );

            let log_entry_ids = (start..self.entries.len())
                .map(LogEntryId::from)
                .collect::<Vec<_>>();

            for log_entry_id in &log_entry_ids {
                self.acknowledgeable_server_ids.insert(
                    usize::from(*log_entry_id),
                    acknowledgeable_server_ids.clone(),
                );
                self.acknowledged_server_ids
                    .insert(usize::from(*log_entry_id), HashSet::default());
            }

            log_entry_ids
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

        pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
            match usize::from(self.next_committing_log_entry_id) {
                0 => None,
                next_committing_log_entry_id => {
                    Some(LogEntryId::from(next_committing_log_entry_id - 1))
                }
            }
        }

        pub fn push(&mut self, log_entry: LogEntry<T>) {
            let id = usize::from(log_entry.id());

            if self.entries.len() < id + 1 {
                self.entries.resize_with(id + 1, || None);
            }

            self.entries.insert(id, Some(log_entry));
        }

        pub fn commit(&mut self, log_entry_id: LogEntryId, term: LogTerm) -> Vec<LogEntryId> {
            let mut missing_log_entry_ids = Vec::default();

            let range = (usize::from(self.next_committing_log_entry_id)
                ..usize::from(log_entry_id) + 1)
                .map(LogEntryId::from);

            for log_entry_id in range {
                match self.entries.get_mut(usize::from(log_entry_id)) {
                    None | Some(None) => {
                        missing_log_entry_ids.push(log_entry_id);
                    }
                    Some(Some(log_entry)) if log_entry.term() < term => {
                        missing_log_entry_ids.push(log_entry_id);
                    }
                    Some(Some(_)) => {
                        if self.next_committing_log_entry_id == log_entry_id {
                            self.next_committing_log_entry_id = LogEntryId::from(
                                usize::from(self.next_committing_log_entry_id) + 1,
                            );
                        }
                    }
                }
            }

            missing_log_entry_ids
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub enum LogType<T> {
        Leading(LogLeadingType<T>),
        Following(LogFollowingType<T>),
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub struct LogIdTag;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub type LogId = TaggedType<usize, LogIdTag>;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    pub struct Log<T> {
        id: LogId,
        r#type: LogType<T>,
    }

    impl<T> Log<T> {
        pub fn id(&self) -> LogId {
            self.id
        }

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

    pub type NoneLog = Log<()>;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Default)]
    pub struct LogManager {
        logs: Vec<Option<Arc<dyn Any>>>,
    }

    impl LogManager {
        pub fn get<T>(&self, id: LogId) -> Option<Arc<RwLock<Log<T>>>>
        where
            T: 'static,
        {
            let log = match self.logs.get(usize::from(id)) {
                Some(None) | None => return None,
                Some(Some(log)) => log,
            };

            let any = unsafe { &*Arc::into_raw(log.clone()) as &dyn Any };

            //println!("ello {} {}", std::any::type_name::<T>(), std::any::type_name::<RwLockT//>);

            any.downcast_ref::<RwLock<Log<T>>>()
                .map(|reference| unsafe { Arc::from_raw(reference as *const RwLock<Log<T>>) })
        }

        /*pub fn get_or_create<T>(&mut self) -> &Arc<RwLock<Log<T>>>
        where
            T: Default + 'static,
        {
            None
        }*/
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_name() {
            let mut log_manager = LogManager::default();
            log_manager.logs.push(Some(Arc::from(RwLock::new(NoneLog {
                id: LogId::from(0),
                r#type: LogType::Leading(LogLeadingType::default()),
            }))));

            assert!(log_manager.get::<()>(LogId::from(0)).is_some());
        }
    }
}*/
