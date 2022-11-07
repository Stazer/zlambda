use std::error;
use std::fmt::{self, Debug, Display, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub const MODULES_SYMBOL: &[u8] = b"modules";

////////////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Module: Debug {
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
