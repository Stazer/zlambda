use crate::library::Library;
use crate::module::{Module, ModuleVersion};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ModuleManager {
    name_version_lookup: HashMap<String, HashMap<ModuleVersion, Arc<Box<dyn Module>>>>,
    modules: Vec<Arc<Box<dyn Module>>>,
    libraries: Vec<Library>,
}

impl ModuleManager {
    pub async fn load(&self, _path: &Path) {}

    pub async fn dispatch(&self, _name: &String, _version: &Option<ModuleVersion>) {}
}
