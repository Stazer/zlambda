#[derive(Debug)]
pub enum LogError {
    NotExisting,
    NotAcknowledgeable,
    AlreadyAcknowledged,
}
