use crate::module::{
    LoadModuleError, Module, ModuleUnloadEventInput, ModuleId, ModuleLoadEventInput,
    ModuleLoadEventOutput, UnloadModuleError,
};
use crate::server::ServerHandle;
use std::any::Any;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct ModuleManager {
    server: ServerHandle,
    modules: Vec<Option<Arc<dyn Module>>>,
}

impl ModuleManager {
    pub fn new(server: ServerHandle) -> Self {
        Self {
            server,
            modules: Vec::default(),
        }
    }

    pub fn get<T>(&self, id: ModuleId) -> Option<Arc<T>>
    where
        T: Module + 'static,
    {
        let module = match self.modules.get(id) {
            None | Some(None) => return None,
            Some(Some(module)) => module,
        };

        let any = unsafe { &*Arc::into_raw(module.clone()) as &dyn Any };

        any.downcast_ref::<T>()
            .map(|reference| unsafe { Arc::from_raw(reference as *const T) })
    }

    pub async fn load<T>(&mut self, module: T) -> Result<ModuleId, LoadModuleError>
    where
        T: Into<Arc<dyn Module>>,
    {
        let module_id = self.modules.len();
        let module = module.into();
        self.modules.push(Some(module.clone()));

        module
            .on_load(ModuleLoadEventInput::new(
                module_id,
                self.server.clone(),
            ))
            .await;

        Ok(module_id)
    }

    pub async fn unload(&mut self, module_id: ModuleId) -> Result<(), UnloadModuleError> {
        let module = match self.modules.get_mut(module_id) {
            None | Some(None) => return Err(UnloadModuleError::ModuleNotFound),
            Some(module) => module.take().ok_or(UnloadModuleError::ModuleNotFound)?,
        };

        module
            .on_unload(ModuleUnloadEventInput::new(self.server.clone()))
            .await;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use crate::module::{
        Module, ModuleUnloadEventInput, ModuleUnloadEventOutput, ModuleLoadEventInput,
        ModuleLoadEventOutput, ModuleManager, UnloadModuleError,
    };
    use tokio::sync::mpsc::{channel, Sender};

    #[tokio::test]
    async fn test_load_ok() {
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        assert!(ModuleManager::default().load(TestModule {}).await.is_ok())
    }

    #[tokio::test]
    async fn test_load_triggers_on_load() {
        struct TestModule {
            sender: Sender<()>,
        }

        #[async_trait::async_trait]
        impl Module for TestModule {
            async fn on_load(
                &self,
                _event: ModuleLoadEventInput,
            ) -> ModuleLoadEventOutput {
                self.sender.send(()).await.unwrap()
            }
        }

        let (sender, mut receiver) = channel(1);

        ModuleManager::default()
            .load(TestModule { sender })
            .await
            .unwrap();

        assert!(receiver.recv().await.is_some())
    }

    #[tokio::test]
    async fn test_get_existing() {
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::default();
        let module_id = manager.load(TestModule {}).await.unwrap();

        assert!(manager.get::<TestModule>(module_id).is_some())
    }

    #[tokio::test]
    async fn test_get_not_existing_index() {
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::default();
        let module_id = manager.load(TestModule {}).await.unwrap();

        assert!(manager.get::<TestModule>(module_id + 1).is_none())
    }

    #[tokio::test]
    async fn test_get_not_existing_type() {
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        struct TestModule2 {}

        #[async_trait::async_trait]
        impl Module for TestModule2 {}

        let mut manager = ModuleManager::default();
        let module_id = manager.load(TestModule {}).await.unwrap();

        assert!(manager.get::<TestModule2>(module_id).is_none())
    }

    #[tokio::test]
    async fn test_unload_ok() {
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::default();
        let module_id = manager.load(TestModule {}).await.unwrap();

        assert!(manager.unload(module_id).await.is_ok())
    }

    #[tokio::test]
    async fn test_unload_not_existing_index() {
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::default();
        let module_id = manager.load(TestModule {}).await.unwrap();

        assert!(manager.unload(module_id + 1).await == Err(UnloadModuleError::ModuleNotFound))
    }

    #[tokio::test]
    async fn test_unload_triggers_on_unload() {
        struct TestModule {
            sender: Sender<()>,
        }

        #[async_trait::async_trait]
        impl Module for TestModule {
            async fn on_unload(
                &self,
                _event: ModuleUnloadEventInput,
            ) -> ModuleUnloadEventOutput {
                self.sender.send(()).await.unwrap()
            }
        }

        let (sender, mut receiver) = channel(1);

        let mut manager = ModuleManager::default();
        let module_id = manager.load(TestModule { sender }).await.unwrap();
        manager.unload(module_id).await.unwrap();

        assert!(receiver.recv().await.is_some())
    }
}
