use crate::module::{
    DispatchModuleEventError, DispatchModuleEventInput, DispatchModuleEventOutput,
    ModuleEventListener,
};
use async_ffi::{BorrowingFfiFuture, FutureExt};
use tokio::runtime::Handle;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait ModuleEventHandler {
    fn dispatch(
        &self,
        handle: Handle,
        input: DispatchModuleEventInput,
    ) -> BorrowingFfiFuture<Result<DispatchModuleEventOutput, DispatchModuleEventError>>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DefaultModuleEventHandler {
    listener: Box<dyn ModuleEventListener>,
}

impl DefaultModuleEventHandler {
    pub fn new(listener: Box<dyn ModuleEventListener>) -> Self {
        Self { listener }
    }
}

impl ModuleEventHandler for DefaultModuleEventHandler {
    fn dispatch(
        &self,
        handle: Handle,
        input: DispatchModuleEventInput,
    ) -> BorrowingFfiFuture<Result<DispatchModuleEventOutput, DispatchModuleEventError>> {
        let future = self.listener.dispatch(input);

        async move {
            let _enter = handle.enter();
            future.await
        }
        .into_ffi()
    }
}
