use std::error::Error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeClientAction {
    Continue,
    ConnectionClosed,
    Stop,
    Error(Box<dyn Error + Send>),
}
