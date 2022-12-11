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

    pub fn load(id: ModuleId, path: &Path) -> Result<Self, LoadModuleError> {
        let _library = unsafe { Library::new(path)? };

        let event_handler = unsafe {
            _library.get::<unsafe extern "C" fn() -> Box<dyn ModuleEventHandler>>(
                MODULE_EVENT_HANDLER_SYMBOL,
            )?()
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
