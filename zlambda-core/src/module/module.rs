use crate::module::{
    ModuleCommitEventInput, ModuleCommitEventOutput, ModuleDispatchEventInput,
    ModuleDispatchEventOutput, ModuleUnloadEventInput, ModuleUnloadEventOutput,
    ModuleLoadEventInput, ModuleLoadEventOutput, ModuleShutdownEventInput,
    ModuleShutdownEventOutput, ModuleStartupEventInput, ModuleStartupEventOutput,
};
use std::any::Any;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait::async_trait]
pub trait Module: Any + Send + Sync + 'static {
    async fn on_startup(&self, _event: ModuleStartupEventInput) -> ModuleStartupEventOutput {}

    async fn on_shutdown(&self, _event: ModuleShutdownEventInput) -> ModuleShutdownEventOutput {}

    async fn on_load(
        &self,
        _event: ModuleLoadEventInput,
    ) -> ModuleLoadEventOutput {
    }

    async fn on_unload(&self, _event: ModuleUnloadEventInput) -> ModuleUnloadEventOutput {}

    async fn on_dispatch(&self, _event: ModuleDispatchEventInput) -> ModuleDispatchEventOutput {}

    async fn on_commit(&self, _event: ModuleCommitEventInput) -> ModuleCommitEventOutput {}
}

impl dyn Module {
    pub fn r#as<T>(self: &Arc<dyn Module>) -> Option<Arc<T>>
    where
        T: Module,
    {
        let any = unsafe { &*Arc::into_raw(self.clone()) as &dyn Any };

        any.downcast_ref::<T>()
            .map(|reference| unsafe { Arc::from_raw(reference as *const T) })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::Module;
    use std::sync::Arc;

    ////////////////////////////////////////////////////////////////////////////////////////////////

    struct EmptyModule {}

    #[async_trait::async_trait]
    impl Module for EmptyModule {}

    ////////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_as_some() {
        let module: Arc<dyn Module> = Arc::from(EmptyModule{});
        assert!(module.r#as::<EmptyModule>().is_some())
    }

    #[test]
    fn test_as_none() {
    struct EmptyModule2 {}

    #[async_trait::async_trait]
    impl Module for EmptyModule2 {}
        let module: Arc<dyn Module> = Arc::from(EmptyModule{});
        assert!(module.r#as::<EmptyModule2>().is_none())
    }
}
