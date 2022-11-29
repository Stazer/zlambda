mod error;
mod event;
mod id;
mod manager;
mod symbol;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use error::*;
pub use event::*;
pub use id::*;
pub use manager::*;
pub use symbol::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;
/*use crate::event::{
    CreateDispatchPayloadEvent, CreateDispatchPayloadEventResult, DispatchEvent,
    DispatchEventResult,
};*/
use libloading::Library;

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
    event_listener: Box<dyn ModuleEventListener>,
    _library: Library,
}

impl Module {
    pub fn load(id: ModuleId, path: &Path) -> Result<Self, LoadModuleError> {
        let _library = unsafe { Library::new(path)? };

        let event_listener = unsafe {
            _library.get::<unsafe extern "C" fn() -> Box<dyn ModuleEventListener>>(
                MODULE_EVENT_LISTENER_SYMBOL,
            )?()
        };

        Ok(Self {
            id,
            event_listener,
            _library,
        })
    }

    pub fn event_listener(&self) -> &dyn ModuleEventListener {
        &*self.event_listener
    }
}
