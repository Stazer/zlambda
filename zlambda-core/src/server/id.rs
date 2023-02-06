/*use serde::{Deserialize, Serialize};
use std::fmt::{Display, self, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Deserialize, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct ServerId(usize);

impl Display for ServerId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl From<usize> for ServerId {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<ServerId> for usize {
    fn from(server_id: ServerId) -> Self {
        server_id.0
    }
}

impl ServerId {
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}*/

pub type ServerId = usize;
