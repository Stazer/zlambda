use crate::common::module::{
    LoadModuleError, Module, ModuleId, UnloadModuleError,
};
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IntoArcModule
{
    fn into_arc_module(self) -> Arc<dyn Module>;
}

impl<T> IntoArcModule for T
where
    T: Module + 'static
{
    fn into_arc_module(self) -> Arc<dyn Module> {
        Arc::from(self)
    }
}

impl IntoArcModule for Box<dyn Module>
{
    fn into_arc_module(self) -> Arc<dyn Module> {
        Arc::from(self)
    }
}

impl IntoArcModule for Arc<dyn Module> {
    fn into_arc_module(self) -> Arc<dyn Module> {
        self
    }
}

impl IntoArcModule for &Arc<dyn Module> {
    fn into_arc_module(self) -> Arc<dyn Module> {
        self.clone()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleManager<T>
where
    T: ?Sized,
{
    modules: Vec<Option<Arc<T>>>,
}

impl<T> Default for ModuleManager<T>
where
    T: ?Sized,
{
    fn default() -> Self {
        Self {
            modules: Vec::default(),
        }
    }
}

impl<T> ModuleManager<T>
where
    T: ?Sized,
{
    pub fn get(&self, id: ModuleId) -> Option<&Arc<T>> {
        match self.modules.get(id) {
            None | Some(None) => None,
            Some(Some(ref module)) => Some(module),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<T>> {
        self.modules.iter().flatten()
    }

    pub fn load(&mut self, module: Arc<T>) -> Result<ModuleId, LoadModuleError>
    {
        let module_id = self.modules.len();
        self.modules.push(Some(module));

        Ok(module_id)
    }

    pub fn unload(&mut self, module_id: ModuleId) -> Result<(), UnloadModuleError> {
        match self.modules.get_mut(module_id) {
            None | Some(None) => Err(UnloadModuleError::ModuleNotFound),
            Some(module) => module.take().map(|_| ()).ok_or(UnloadModuleError::ModuleNotFound),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use crate::module::{
        Module, ModuleUnloadEventInput, ModuleUnloadEventOutput, ModuleLoadEventInput,
        ModuleLoadEventOutput, ModuleManager, UnloadModuleError,
    };

    ////////////////////////////////////////////////////////////////////////////////////////////////

    struct EmptyModule {}

    #[async_trait::async_trait]
    impl Module for EmptyModule {}

    ////////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_load_ok() {
        assert!(ModuleManager::default().load(EmptyModule {}).is_ok())
    }

    #[test]
    fn test_get_some() {
        let mut manager = ModuleManager::default();
        let module_id = manager.load(EmptyModule {}).unwrap();

        assert!(manager.get(module_id).is_some())
    }

    #[test]
    fn test_get_none() {
        let mut manager = ModuleManager::default();
        let module_id = manager.load(EmptyModule {}).unwrap();

        assert!(manager.get(module_id + 1).is_none())
    }

    #[test]
    fn test_unload_ok() {
        let mut manager = ModuleManager::default();
        let module_id = manager.load(EmptyModule {}).unwrap();

        assert!(manager.unload(module_id).is_ok())
    }

    #[test]
    fn test_unload_ok_get_none() {
        let mut manager = ModuleManager::default();
        let module_id = manager.load(EmptyModule {}).unwrap();

        manager.unload(module_id).unwrap();

        assert!(manager.get(module_id).is_none())
    }

    #[test]
    fn test_unload_err_module_not_existing() {
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::default();
        let module_id = manager.load(TestModule {}).unwrap();

        assert!(manager.unload(module_id + 1) == Err(UnloadModuleError::ModuleNotFound))
    }
}
