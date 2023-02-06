use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ServerClientId(usize);

impl Display for ServerClientId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl From<usize> for ServerClientId {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<ServerClientId> for usize {
    fn from(server_id: ServerClientId) -> Self {
        server_id.0
    }
}

impl FromStr for ServerClientId {
    type Err = <usize as FromStr>::Err;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        <usize as FromStr>::from_str(string).map(Self::from)
    }
}

impl ServerClientId {
    pub fn new(value: usize) -> Self {
        Self(value)
    }
}
