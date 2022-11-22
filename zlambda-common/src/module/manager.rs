use crate::library::Library;
use crate::module::{Module};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ModuleManager {
    modules: Vec<Arc<Box<dyn Module>>>,
    libraries: Vec<Library>,
}

impl ModuleManager {
    pub async fn load(&self, _path: &Path) {}
}
