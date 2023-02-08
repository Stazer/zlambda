use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Module: Any + Debug + Send + Sync {}

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

    #[derive(Debug)]
    struct EmptyModule {}

    #[async_trait::async_trait]
    impl Module for EmptyModule {}

    ////////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_as_some() {
        let module: Arc<dyn Module> = Arc::from(EmptyModule {});
        assert!(module.r#as::<EmptyModule>().is_some())
    }

    #[test]
    fn test_as_none() {
        #[derive(Debug)]
        struct EmptyModule2 {}

        #[async_trait::async_trait]
        impl Module for EmptyModule2 {}
        let module: Arc<dyn Module> = Arc::from(EmptyModule {});
        assert!(module.r#as::<EmptyModule2>().is_none())
    }
}
