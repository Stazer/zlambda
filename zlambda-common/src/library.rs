use crate::module::{Module, ReadModulesError, MODULE_SYMBOL};
use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum LoadLibraryError {
    LibloadingError(libloading::Error),
}

impl Debug for LoadLibraryError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::LibloadingError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for LoadLibraryError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::LibloadingError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for LoadLibraryError {}

impl From<libloading::Error> for LoadLibraryError {
    fn from(error: libloading::Error) -> Self {
        Self::LibloadingError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Library {
    module: Box<dyn Module>,
    library: libloading::Library,
}

impl Library {
    pub fn load(path: &Path) -> Result<Self, LoadLibraryError> {
        let library = unsafe { libloading::Library::new(path)? };

        let module =
            unsafe { library.get::<unsafe extern "C" fn() -> Box<dyn Module>>(MODULE_SYMBOL)?() };

        Ok(Self { module, library })
    }

    pub fn module(&self) -> &dyn Module {
        &*self.module
    }
}
