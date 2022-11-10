use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;
use crate::module::{Module, ModuleVersion};
use crate::library::Library;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ModuleManager {
    name_version_lookup: HashMap<String, HashMap<ModuleVersion, Arc<Box<dyn Module>>>>,
    modules: Vec<Arc<Box<dyn Module>>>,
    libraries: Vec<Library>,
}

impl ModuleManager {
    pub async fn load(&self, path: &Path) {
    }

    pub async fn dispatch(&self, name: &String, version: &Option<ModuleVersion>) {

    }
}
