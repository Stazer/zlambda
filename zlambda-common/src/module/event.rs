use crate::async_trait::async_trait;
use postcard::{take_from_bytes, to_allocvec};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt::{self, Debug, Display, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ModuleEventError {
    PostcardError(postcard::Error),
    Custom(Box<dyn error::Error>),
}

impl error::Error for ModuleEventError {}

impl Debug for ModuleEventError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Debug::fmt(error, formatter),
            Self::Custom(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for ModuleEventError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Display::fmt(error, formatter),
            Self::Custom(error) => Display::fmt(error, formatter),
        }
    }
}

impl From<postcard::Error> for ModuleEventError {
    fn from(error: postcard::Error) -> Self {
        Self::PostcardError(error)
    }
}

impl From<Box<dyn error::Error>> for ModuleEventError {
    fn from(error: Box<dyn error::Error>) -> Self {
        Self::Custom(error)
    }
}

impl From<&str> for ModuleEventError {
    fn from(error: &str) -> Self {
        Self::Custom(error.into())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ModuleEventDispatchPayload(Vec<u8>);

impl From<Vec<u8>> for ModuleEventDispatchPayload {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<ModuleEventDispatchPayload> for Vec<u8> {
    fn from(payload: ModuleEventDispatchPayload) -> Self {
        payload.0
    }
}

impl ModuleEventDispatchPayload {
    pub fn new<T>(instance: &T) -> Result<Self, postcard::Error>
    where
        T: Serialize,
    {
        Ok(Self(to_allocvec(instance)?))
    }

    pub fn into_inner<T>(self) -> Result<T, postcard::Error>
    where
        T: DeserializeOwned,
    {
        Ok(take_from_bytes::<T>(&self.0)?.0)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DispatchModuleEventInput {
    payload: ModuleEventDispatchPayload,
}

impl From<DispatchModuleEventInput> for (ModuleEventDispatchPayload,) {
    fn from(input: DispatchModuleEventInput) -> Self {
        (input.payload,)
    }
}

impl DispatchModuleEventInput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DispatchModuleEventOutput {
    payload: ModuleEventDispatchPayload,
}

impl From<DispatchModuleEventOutput> for (ModuleEventDispatchPayload,) {
    fn from(input: DispatchModuleEventOutput) -> Self {
        (input.payload,)
    }
}

impl DispatchModuleEventOutput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type DispatchModuleEventError = ModuleEventError;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ReadModuleEventInput {
    arguments: Vec<String>,
}

impl From<ReadModuleEventInput> for (Vec<String>,) {
    fn from(input: ReadModuleEventInput) -> Self {
        (input.arguments,)
    }
}

impl ReadModuleEventInput {
    pub fn new(arguments: Vec<String>) -> Self {
        Self { arguments }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ReadModuleEventOutput {
    payload: ModuleEventDispatchPayload,
}

impl From<ReadModuleEventOutput> for (ModuleEventDispatchPayload,) {
    fn from(output: ReadModuleEventOutput) -> Self {
        (output.payload,)
    }
}

impl ReadModuleEventOutput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ReadModuleEventError = ModuleEventError;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct WriteModuleEventInput {
    payload: ModuleEventDispatchPayload,
}

impl From<WriteModuleEventInput> for (ModuleEventDispatchPayload,) {
    fn from(input: WriteModuleEventInput) -> Self {
        (input.payload,)
    }
}

impl WriteModuleEventInput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type WriteModuleEventOutput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type WriteModuleEventError = ModuleEventError;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait ModuleEventListener: Send + Sync {
    async fn read(
        &self,
        event: ReadModuleEventInput,
    ) -> Result<ReadModuleEventOutput, ReadModuleEventError>;
    async fn dispatch(
        &self,
        event: DispatchModuleEventInput,
    ) -> Result<DispatchModuleEventOutput, DispatchModuleEventError>;
    async fn write(
        &self,
        event: WriteModuleEventInput,
    ) -> Result<WriteModuleEventOutput, WriteModuleEventError>;
}
