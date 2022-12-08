use crate::async_trait::async_trait;
use postcard::{take_from_bytes, to_allocvec};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter, Debug};
use std::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
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

impl DispatchModuleEventOutput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum DispatchModuleEventError {}

impl error::Error for DispatchModuleEventError {}

impl Debug for  DispatchModuleEventError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl Display for DispatchModuleEventError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

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

pub enum ReadModuleEventError {}

impl error::Error for ReadModuleEventError {}

impl Debug for  ReadModuleEventError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl Display for  ReadModuleEventError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct WriteModuleEventInput {
    payload: ModuleEventDispatchPayload,
}

impl WriteModuleEventInput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type WriteModuleEventOutput = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum WriteModuleEventError {}

impl error::Error for WriteModuleEventError {}

impl Debug for  WriteModuleEventError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl Display for  WriteModuleEventError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

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
