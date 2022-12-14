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
    payload: Vec<u8>,
}

impl From<DispatchModuleEventInput> for (Vec<u8>,) {
    fn from(input: DispatchModuleEventInput) -> Self {
        (input.payload,)
    }
}

impl DispatchModuleEventInput {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DispatchModuleEventOutput {
    payload: Vec<u8>,
}

impl From<DispatchModuleEventOutput> for (Vec<u8>,) {
    fn from(output: DispatchModuleEventOutput) -> Self {
        (output.payload,)
    }
}

impl DispatchModuleEventOutput {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type DispatchModuleEventError = ModuleEventError;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct InitializeModuleEventInput {}

impl InitializeModuleEventInput {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct InitializeModuleEventOutput {
    event_listener: Box<dyn ModuleEventListener>,
}

impl From<InitializeModuleEventOutput> for (Box<dyn ModuleEventListener>,) {
    fn from(output: InitializeModuleEventOutput) -> Self {
        (output.event_listener,)
    }
}

impl InitializeModuleEventOutput {
    pub fn new(event_listener: Box<dyn ModuleEventListener>) -> Self {
        Self { event_listener }
    }

    pub fn event_listener(&self) -> &Box<dyn ModuleEventListener> {
        &self.event_listener
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type InitializeModuleEventError = ModuleEventError;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait ModuleEventListener: Send + Sync {
    async fn initialize(
        input: InitializeModuleEventInput,
    ) -> Result<InitializeModuleEventOutput, DispatchModuleEventError>
    where
        Self: Sized;

    async fn dispatch(
        &self,
        input: DispatchModuleEventInput,
    ) -> Result<DispatchModuleEventOutput, DispatchModuleEventError>;
}
