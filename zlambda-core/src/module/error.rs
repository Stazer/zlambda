#[derive(Debug)]
pub enum LoadModuleError {}


////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Eq, PartialEq)]
pub enum UnloadModuleError {
    ModuleNotFound,
}
