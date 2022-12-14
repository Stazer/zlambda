mod error;
mod event;
mod handler;
mod id;
mod manager;
mod symbol;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use error::*;
pub use event::*;
pub use handler::*;
pub use id::*;
pub use manager::*;
pub use symbol::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use async_ffi::{BorrowingFfiFuture, FfiFuture, FutureExt};

////////////////////////////////////////////////////////////////////////////////////////////////////

use libloading::Library;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum LoadModuleError {
    LibloadingError(libloading::Error),
}

impl Debug for LoadModuleError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::LibloadingError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for LoadModuleError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::LibloadingError(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for LoadModuleError {}

impl From<libloading::Error> for LoadModuleError {
    fn from(error: libloading::Error) -> Self {
        Self::LibloadingError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Module {
    id: ModuleId,
    event_handler: Box<dyn ModuleEventHandler>,
    _library: Library,
}

impl Module {
    fn new(id: ModuleId, event_handler: Box<dyn ModuleEventHandler>, _library: Library) -> Self {
        Self {
            id,
            event_handler,
            _library,
        }
    }

    pub async fn load(id: ModuleId, path: &Path) -> Result<Self, LoadModuleError> {
        let _library = unsafe { Library::new(path)? };

        let event_handler = unsafe {
            let future = _library.get::<unsafe extern "C" fn(
                tokio::runtime::Handle,
            )
                -> ModuleEventListenerCallbackReturn>(
                MODULE_EVENT_LISTENER_SYMBOL
            )?(tokio::runtime::Handle::current());

            future.await.expect("asd")
        };

        Ok(Self::new(id, event_handler, _library))
    }

    pub fn id(&self) -> ModuleId {
        self.id
    }

    pub fn event_handler(&self) -> &dyn ModuleEventHandler {
        &*self.event_handler
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ModuleEventListenerCallbackReturn =
    FfiFuture<Result<Box<dyn ModuleEventHandler>, InitializeModuleEventError>>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! module_event_listener(
    ($type:ty) => {
        #[no_mangle]
        pub extern "C" fn module_event_listener(
            handle: tokio::runtime::Handle,
        ) -> zlambda_common::module::ModuleEventListenerCallbackReturn {
            use zlambda_common::module::{
                FutureExt,
                ModuleEventHandler,
                DefaultModuleEventHandler
            };

            async move {
                let _enter = handle.enter();

                let (event_listener,) = <$type as ModuleEventListener>::initialize(
                    InitializeModuleEventInput::new(),
                ).await?.into();

                Ok(
                    Box::<dyn ModuleEventHandler>::from(
                        Box::new(DefaultModuleEventHandler::new(event_listener))
                    )
                )
            }.into_ffi()
        }
    }
);
