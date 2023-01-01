use crate::algorithm::next_key;
use crate::module::{Module, ModuleId};
use bytes::Bytes;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

////////////////////////////////////////////////////////////////////////////////////////////////////

enum ModuleManagerEntry {
    Loaded(Arc<Module>),
    Loading {
        handle: NamedTempFile,
        writer: BufWriter<File>,
        path: PathBuf,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ModuleManager {
    entries: HashMap<ModuleId, ModuleManagerEntry>,
}

impl ModuleManager {
    pub fn get(&self, id: ModuleId) -> Option<&Arc<Module>> {
        self.entries.get(&id).and_then(|entry| match entry {
            ModuleManagerEntry::Loaded(module) => Some(module),
            ModuleManagerEntry::Loading { .. } => None,
        })
    }

    pub fn initialize(&mut self) -> Result<ModuleId, Box<dyn Error>> {
        let id = next_key(self.entries.keys());

        let handle = NamedTempFile::new()?;
        let path = handle.path().into();

        let writer = BufWriter::new(File::from_std(handle.reopen()?));

        self.entries.insert(
            id,
            ModuleManagerEntry::Loading {
                handle,
                writer,
                path,
            },
        );

        Ok(id)
    }

    pub async fn append(&mut self, id: ModuleId, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
        let writer = match self.entries.get_mut(&id) {
            None => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loaded(_)) => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loading { writer, .. }) => writer,
        };

        writer.write_all(chunk).await?;

        Ok(())
    }

    pub async fn load(&mut self, id: ModuleId) -> Result<(), Box<dyn Error>> {
        let (handle, path) = match self.entries.remove(&id) {
            None => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loaded(_)) => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loading { handle, path, .. }) => (handle, path),
        };

        self.entries.insert(
            id,
            ModuleManagerEntry::Loaded(Arc::new(Module::load(id, &path).await?)),
        );

        handle.close()?;

        Ok(())
    }
}
