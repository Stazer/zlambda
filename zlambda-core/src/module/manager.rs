use crate::module::{
    LoadModuleError, Module, ModuleUnloadEventInput, ModuleId, ModuleLoadEventInput,
    ModuleLoadEventOutput, UnloadModuleError,
};
use crate::server::ServerHandle;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait IntoArcModule {
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

    pub fn get(&self, id: ModuleId) -> Option<&Arc<dyn Module>> {
        match self.modules.get(id) {
            None | Some(None) => None,
            Some(Some(ref module)) => Some(module),
        }
    }

    pub async fn load<T>(&mut self, module: T) -> Result<ModuleId, LoadModuleError>
    where
        T: IntoArcModule + 'static,
    {
        let module_id = self.modules.len();
        let module = module.into_arc_module();
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
    use crate::message::{MessageQueueSender, message_queue};
    use crate::server::ServerHandle;
    use crate::module::{
        Module, ModuleUnloadEventInput, ModuleUnloadEventOutput, ModuleLoadEventInput,
        ModuleLoadEventOutput, ModuleManager, UnloadModuleError,
    };
    use tokio::sync::mpsc::{channel, Sender};

    ////////////////////////////////////////////////////////////////////////////////////////////////

    fn server_handle() -> ServerHandle {
        ServerHandle::new(
            message_queue().0
        )
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    struct EmptyModule {}

    #[async_trait::async_trait]
    impl Module for EmptyModule {}

    ////////////////////////////////////////////////////////////////////////////////////////////////

    #[tokio::test]
    async fn test_load_ok() {
        assert!(ModuleManager::new(server_handle()).load(EmptyModule {}).await.is_ok())
    }

    #[tokio::test]
    async fn test_load_triggers_on_load() {
        struct LoadNotifyModule {
            sender: Sender<()>,
        }

        #[async_trait::async_trait]
        impl Module for LoadNotifyModule {
            async fn on_load(
                &self,
                _event: ModuleLoadEventInput,
            ) -> ModuleLoadEventOutput {
                self.sender.send(()).await.unwrap()
            }
        }

        let (sender, mut receiver) = channel(1);

        ModuleManager::new(server_handle())
            .load(LoadNotifyModule { sender })
            .await
            .unwrap();

        assert!(receiver.recv().await.is_some())
    }

    #[tokio::test]
    async fn test_get_some() {
        let mut manager = ModuleManager::new(server_handle());
        let module_id = manager.load(EmptyModule {}).await.unwrap();

        assert!(manager.get(module_id).is_some())
    }

    #[tokio::test]
    async fn test_get_none() {
        let mut manager = ModuleManager::new(server_handle());
        let module_id = manager.load(EmptyModule {}).await.unwrap();

        assert!(manager.get(module_id + 1).is_none())
    }

    #[tokio::test]
    async fn test_unload_ok() {
        let mut manager = ModuleManager::new(server_handle());
        let module_id = manager.load(EmptyModule {}).await.unwrap();

        assert!(manager.unload(module_id).await.is_ok())
    }

    #[tokio::test]
    async fn test_unload_err_module_not_existing() {
        struct TestModule {}

        #[async_trait::async_trait]
        impl Module for TestModule {}

        let mut manager = ModuleManager::new(server_handle());
        let module_id = manager.load(TestModule {}).await.unwrap();

        assert!(manager.unload(module_id + 1).await == Err(UnloadModuleError::ModuleNotFound))
    }

    #[tokio::test]
    async fn test_unload_triggers_on_unload() {
        struct UnloadNotifyModule {
            sender: Sender<()>,
        }

        #[async_trait::async_trait]
        impl Module for UnloadNotifyModule {
            async fn on_unload(
                &self,
                _event: ModuleUnloadEventInput,
            ) -> ModuleUnloadEventOutput {
                self.sender.send(()).await.unwrap()
            }
        }

        let (sender, mut receiver) = channel(1);

        let mut manager = ModuleManager::new(server_handle());
        let module_id = manager.load(UnloadNotifyModule { sender }).await.unwrap();
        manager.unload(module_id).await.unwrap();

        assert!(receiver.recv().await.is_some())
    }
}
