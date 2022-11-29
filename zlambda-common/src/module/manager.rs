use crate::algorithm::next_key;
use crate::module::{Module, ModuleId};
use std::collections::HashMap;
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////////////////////////

enum ModuleManagerEntry {
    Loaded(Module),
    Loading { id: ModuleId },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ModuleManager {
    entries: HashMap<ModuleId, ModuleManagerEntry>,
}

impl ModuleManager {
    pub fn get(&self, id: ModuleId) -> Option<&Module> {
        self.entries.get(&id).and_then(|entry| match entry {
            ModuleManagerEntry::Loaded(module) => Some(module),
            ModuleManagerEntry::Loading { .. } => None,
        })
    }

    pub fn initialize(&mut self) -> ModuleId {
        let id = next_key(self.entries.keys());

        self.entries.insert(id, ModuleManagerEntry::Loading { id });

        id
    }

    pub fn append(&mut self, id: ModuleId) {
    }

    pub async fn load(&mut self, _path: &Path) -> ModuleId {
        0
    }
}
