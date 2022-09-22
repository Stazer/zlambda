use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Module {
}

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

pub enum ReadModulesError {
    LibloadingError(libloading::Error),
}

impl Debug for ReadModulesError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::LibloadingError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for ReadModulesError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::LibloadingError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for ReadModulesError {}

impl From<libloading::Error> for ReadModulesError {
    fn from(error: libloading::Error) -> Self {
        Self::LibloadingError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Library {
    handle: libloading::Library,
}

impl Library {
    pub fn load(path: &Path) -> Result<Self, LoadLibraryError> {
        Ok(Self {
            handle: unsafe { libloading::Library::new(path)? },
        })
    }

    pub fn modules(&self) -> Result<Vec<Box<dyn Module>>, ReadModulesError> {
        Ok(unsafe {
            self.handle
                .get::<unsafe extern "C" fn() -> Vec<Box<dyn Module>>>(b"modules")?()
        })
    }
}
