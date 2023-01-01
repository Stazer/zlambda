use crate::module::{Module, ModuleId};
use std::error::Error;
use std::mem::replace;
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
    entries: Vec<Option<ModuleManagerEntry>>,
}

impl ModuleManager {
    pub fn get(&self, id: ModuleId) -> Option<&Arc<Module>> {
        self.entries.get(id as usize).and_then(|entry| match entry {
            Some(ModuleManagerEntry::Loaded(module)) => Some(module),
            Some(ModuleManagerEntry::Loading { .. }) | None => None,
        })
    }

    pub fn initialize(&mut self) -> Result<ModuleId, Box<dyn Error>> {
        let id = self.entries.len();

        let handle = NamedTempFile::new()?;
        let path = handle.path().into();

        let writer = BufWriter::new(File::from_std(handle.reopen()?));

        self.entries.push(Some(ModuleManagerEntry::Loading {
            handle,
            writer,
            path,
        }));

        Ok(id as ModuleId)
    }

    pub async fn append(&mut self, id: ModuleId, chunk: &[u8]) -> Result<(), Box<dyn Error>> {
        let writer = match self.entries.get_mut(id as usize) {
            None | Some(None) => return Err("Module not found".into()),
            Some(Some(ModuleManagerEntry::Loaded(_))) => return Err("Module already loaded".into()),
            Some(Some(ModuleManagerEntry::Loading { writer, .. })) => writer,
        };

        writer.write_all(chunk).await?;

        Ok(())
    }

    pub async fn load(&mut self, id: ModuleId) -> Result<(), Box<dyn Error>> {
        let entry = match self.entries.get_mut(id as usize) {
            None => return Err("Module not found".into()),
            Some(entry) => entry.take(),
        };

        let (handle, path) = match entry {
            None => return Err("Module not found".into()),
            Some(ModuleManagerEntry::Loaded(_)) => return Err("Module already loaded found".into()),
            Some(ModuleManagerEntry::Loading { handle, path, .. }) => (handle, path),
        };

        let module = Arc::new(Module::load(id, &path).await?);

        handle.close()?;

        match self.entries.get_mut(id as usize) {
            None => return Err("Module not found".into()),
            Some(entry) => replace(entry, Some(ModuleManagerEntry::Loaded(module))),
        };

        Ok(())
    }
}
