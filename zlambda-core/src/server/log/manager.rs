use crate::server::{Log, LogId};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LogManager {
    logs: Vec<Option<Log>>,
}

impl LogManager {
    pub fn get(&self, id: LogId) -> Option<&Log> {
        match self.logs.get(usize::from(id)) {
            None | Some(None) => None,
            Some(Some(log)) => Some(log),
        }
    }

    pub fn get_mut(&mut self, id: LogId) -> Option<&mut Log> {
        match self.logs.get_mut(usize::from(id)) {
            None | Some(None) => None,
            Some(Some(log)) => Some(log),
        }
    }

    pub fn insert(&mut self, log: Log) {
        let log_id = usize::from(log.id());

        if log_id >= self.logs.len() {
            self.logs.resize_with(log_id + 1, || None);
        }

        *self.logs.get_mut(log_id).expect("existing logs") = Some(log);
    }

    pub fn len(&self) -> usize {
        self.logs.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Log> {
        self.logs.iter().flatten()
    }
}
