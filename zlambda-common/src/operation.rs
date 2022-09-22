use serde::{Deserialize, Serialize};
use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub enum OperationRequestType {
    NoOperation,
    Ping,
    LoadModule { path: PathBuf },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct OperationRequest {
    r#type: OperationRequestType,
}

impl OperationRequest {
    pub fn new(r#type: OperationRequestType) -> Self {
        Self { r#type }
    }

    pub fn r#type(&self) -> &OperationRequestType {
        &self.r#type
    }

    pub fn type_mut(&mut self) -> &mut OperationRequestType {
        &mut self.r#type
    }

    pub fn set_type(&mut self, r#type: OperationRequestType) {
        self.r#type = r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct OperationResponse {}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Operation {}

impl Operation for OperationRequest {}
impl Operation for OperationResponse {}
