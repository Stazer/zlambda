use serde::{Deserialize, Serialize};
use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SimpleError {
    message: String,
}

impl Debug for SimpleError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{}", self.message)
    }
}

impl Display for SimpleError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{}", self.message)
    }
}

impl error::Error for SimpleError {}

impl SimpleError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RecoveryError {
    FollowerAlreadyOnline,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FollowerRegistrationNotALeaderError {
    leader_address: SocketAddr,
}

impl From<FollowerRegistrationNotALeaderError> for (SocketAddr,) {
    fn from(error: FollowerRegistrationNotALeaderError) -> Self {
        (error.leader_address,)
    }
}

impl From<FollowerRegistrationNotALeaderError> for FollowerRegistrationError {
    fn from(error: FollowerRegistrationNotALeaderError) -> Self {
        Self::NotALeader(error)
    }
}

impl FollowerRegistrationNotALeaderError {
    pub fn new(leader_address: SocketAddr) -> Self {
        Self { leader_address }
    }

    pub fn leader_address(&self) -> &SocketAddr {
        &self.leader_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerRegistrationError {
    NotALeader(FollowerRegistrationNotALeaderError),
}
