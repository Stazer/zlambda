#[derive(Debug)]
pub enum DispatchEventOperation {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DispatchEvent {
    operation: DispatchEventOperation,
}