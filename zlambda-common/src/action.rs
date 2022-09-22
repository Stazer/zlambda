use serde::{Deserialize, Serialize};
use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize)]
pub enum ActionType {
    NoOperation,
    Ping,
    LoadModule { path: PathBuf },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize)]
pub struct Action {
    r#type: ActionType,
}

impl Action {
    pub fn new(r#type: ActionType) -> Self {
        Self { r#type }
    }

    pub fn r#type(&self) -> &ActionType {
        &self.r#type
    }

    pub fn type_mut(&mut self) -> &mut ActionType {
        &mut self.r#type
    }

    pub fn set_type(&mut self, r#type: ActionType) {
        self.r#type = r#type
    }
}
