use crate::algorithm::next_key;
use crate::module::{Module, ModuleId};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

////////////////////////////////////////////////////////////////////////////////////////////////////

enum ModuleManagerEntry {
    Loaded(Module),
    Loading { file: File, path: PathBuf },
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

    pub fn initialize(&mut self) -> Result<ModuleId, Box<dyn Error>> {
        let id = next_key(self.entries.keys());

        let tempfile = NamedTempFile::new()?;
        let path = tempfile.path().into();
        let file = File::from_std(tempfile.into_file());

        self.entries
            .insert(id, ModuleManagerEntry::Loading { file, path });

        Ok(id)
    }

    pub async fn append(&mut self, id: ModuleId, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
        let file = match self.entries.get_mut(&id) {
            None => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loaded(_)) => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loading { file, .. }) => file,
        };

        file.write_all(chunk).await?;

        Ok(())
    }

    pub async fn load(&mut self, id: ModuleId) -> Result<(), Box<dyn Error>> {
        let path = match self.entries.remove(&id) {
            None => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loaded(_)) => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loading { path, .. }) => path,
        };

        self.entries
            .insert(id, ModuleManagerEntry::Loaded(Module::load(id, &path)?));

        Ok(())
    }
}
