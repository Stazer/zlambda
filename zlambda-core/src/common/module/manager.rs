use crate::common::module::{LoadModuleError, Module, ModuleId, UnloadModuleError};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleManager<T>
where
    T: ?Sized + 'static,
{
    modules: Vec<Option<Arc<T>>>,
    type_id_lookup: HashMap<TypeId, Arc<T>>,
}

impl<T> Default for ModuleManager<T>
where
    T: ?Sized + 'static,
{
    fn default() -> Self {
        Self {
            modules: Vec::default(),
            type_id_lookup: HashMap::default(),
        }
    }
}

impl<T> ModuleManager<T>
where
    T: ?Sized + 'static,
{
    pub fn get_by_module_id(&self, id: ModuleId) -> Option<&Arc<T>> {
        match self.modules.get(usize::from(id)) {
            None | Some(None) => None,
            Some(Some(ref module)) => Some(module),
        }
    }

    pub fn get_by_type_id(&self, type_id: TypeId) -> Option<&Arc<T>> {
        self.type_id_lookup.get(&type_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ModuleId, &Arc<T>)> {
        self.modules
            .iter()
            .enumerate()
            .flat_map(|(module_id, module)| match module {
                Some(module) => Some((ModuleId::from(module_id), module)),
                None => None,
            })
    }

    pub fn load(&mut self, module: Arc<T>) -> Result<ModuleId, LoadModuleError> {
        let module_id = self.modules.len();
        let type_id = (*module).type_id();

        if self.type_id_lookup.get(&type_id).is_none() {
            self.type_id_lookup.insert(type_id, module.clone());
        }

        self.modules.push(Some(module));

        Ok(ModuleId::from(module_id))
    }

    pub fn unload(&mut self, module_id: ModuleId) -> Result<(), UnloadModuleError> {
        match self.modules.get_mut(usize::from(module_id)) {
            None | Some(None) => Err(UnloadModuleError::ModuleNotFound),
            Some(module) => module
                .take()
                .map(|_| ())
                .ok_or(UnloadModuleError::ModuleNotFound),
        }
    }
}

impl ModuleManager<dyn Module> {
    pub fn get_by_type<S>(&self) -> Option<Arc<S>>
    where
        S: Module,
    {
        self.get_by_type_id(TypeId::of::<S>())
            .map(|module| module.r#as::<S>())
            .flatten()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use crate::common::module::{Module, ModuleId, ModuleManager, UnloadModuleError};
    use std::sync::Arc;

    ////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Debug)]
    struct EmptyModule {}

    #[async_trait::async_trait]
    impl Module for EmptyModule {}

    ////////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_load_ok() {
        assert!(ModuleManager::default()
            .load(Arc::from(EmptyModule {}))
            .is_ok())
    }

    #[test]
    fn test_get_by_module_id_some() {
        let mut manager = ModuleManager::default();
        let module_id = manager.load(Arc::from(EmptyModule {})).unwrap();

        assert!(manager.get_by_module_id(module_id).is_some())
    }

    #[test]
    fn test_get_by_module_id_none() {
        let mut manager = ModuleManager::default();
        let module_id = manager.load(Arc::from(EmptyModule {})).unwrap();

        assert!(manager
            .get_by_module_id(ModuleId::from(usize::from(module_id) + 1))
            .is_none())
    }

    #[test]
    fn test_unload_ok() {
        let mut manager = ModuleManager::default();
        let module_id = manager.load(Arc::from(EmptyModule {})).unwrap();

        assert!(manager.unload(module_id).is_ok())
    }

    #[test]
    fn test_unload_ok_get_none() {
        let mut manager = ModuleManager::default();
        let module_id = manager.load(Arc::from(EmptyModule {})).unwrap();

        manager.unload(module_id).unwrap();

        assert!(manager.get_by_module_id(module_id).is_none())
    }

    #[test]
    fn test_unload_err_module_not_existing() {
        #[derive(Debug)]
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::default();
        let module_id = manager.load(Arc::from(TestModule {})).unwrap();

        assert!(
            manager.unload(ModuleId::from(usize::from(module_id) + 1))
                == Err(UnloadModuleError::ModuleNotFound)
        )
    }

    #[test]
    fn test_get_by_type_some() {
        #[derive(Debug)]
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::<dyn Module>::default();
        manager.load(Arc::from(EmptyModule {})).unwrap();
        manager.load(Arc::from(TestModule {})).unwrap();

        assert!(manager.get_by_type::<EmptyModule>().is_some())
    }

    #[test]
    fn test_get_first_by_type_some() {
        #[derive(Debug)]
        struct TestModule {
            value: usize,
        }

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let value = 91195;

        let mut manager = ModuleManager::<dyn Module>::default();
        manager.load(Arc::from(EmptyModule {})).unwrap();
        manager.load(Arc::from(TestModule { value })).unwrap();
        manager
            .load(Arc::from(TestModule { value: value - 1 }))
            .unwrap();

        assert!(
            manager
                .get_by_type::<TestModule>()
                .map(|module| module.value)
                .unwrap_or_default()
                == value
        )
    }

    #[test]
    fn test_get_by_type_none() {
        #[derive(Debug)]
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::<dyn Module>::default();
        manager.load(Arc::from(TestModule {})).unwrap();

        assert!(manager.get_by_type::<EmptyModule>().is_none())
    }
}
