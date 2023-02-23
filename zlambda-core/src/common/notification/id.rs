use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(
    Copy, Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct NotificationId(usize);

impl Display for NotificationId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl From<usize> for NotificationId {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<NotificationId> for usize {
    fn from(server_id: NotificationId) -> Self {
        server_id.0
    }
}

impl FromStr for NotificationId {
    type Err = <usize as FromStr>::Err;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        <usize as FromStr>::from_str(string).map(Self::from)
    }
}

impl NotificationId {
    fn new(value: usize) -> Self {
        Self(value)
    }
}
