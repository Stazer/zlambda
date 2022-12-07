use crate::async_trait::async_trait;
use postcard::{take_from_bytes, to_allocvec};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleEventDispatchPayload(Vec<u8>);

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
pub struct RunModuleEventInput {
    arguments: Vec<String>,
}

impl From<RunModuleEventInput> for (Vec<String>,) {
    fn from(input: RunModuleEventInput) -> Self {
        (input.arguments,)
    }
}

impl RunModuleEventInput {
    pub fn new(arguments: Vec<String>) -> Self {
        Self { arguments }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RunModuleEventOutput {
    payload: ModuleEventDispatchPayload,
}

impl RunModuleEventOutput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ExecuteModuleEventInput {
    payload: ModuleEventDispatchPayload,
}

impl ExecuteModuleEventInput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ExecuteModuleEventOutput {
    payload: ModuleEventDispatchPayload,
}

impl ExecuteModuleEventOutput {
    pub fn new(payload: ModuleEventDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait ModuleEventListener: Send + Sync {
    async fn run(&self, event: RunModuleEventInput) -> RunModuleEventOutput;
    async fn execute(&self, event: ExecuteModuleEventInput) -> ExecuteModuleEventOutput;
}
