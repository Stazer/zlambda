pub mod channel;
pub mod error;
pub mod log;
pub mod message;
pub mod node;

pub mod dispatch {
    pub type DispatchId = usize;
}

pub mod module {
    pub type ModuleId = usize;
}

pub mod term {
    pub type Term = usize;
}
