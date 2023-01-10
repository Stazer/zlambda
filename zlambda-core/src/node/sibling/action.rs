use std::error::Error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeSiblingAction {
    Continue,
    Stop,
    Error(Box<dyn Error + Send>),
}
