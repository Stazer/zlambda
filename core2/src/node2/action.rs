use std::error::Error;
use std::io;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeAction {
    Continue,
    Stop,
    Error(Box<dyn Error + Send>),
}

impl From<Box<dyn Error + Send>> for NodeAction {
    fn from(error: Box<dyn Error + Send>) -> Self {
        Self::Error(error)
    }
}

impl From<io::Error> for NodeAction {
    fn from(error: io::Error) -> NodeAction {
        Self::Error(Box::new(error))
    }
}
