use bytes::Bytes;
use postcard::{take_from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

mod candidate_to_candidate;
mod client_to_node;
mod error;
mod follower_to_follower;
mod follower_to_guest;
mod follower_to_leader;
mod guest_to_leader;
mod guest_to_node;
mod leader_to_follower;
mod leader_to_guest;
mod node_to_client;
mod reader;
mod writer;

pub use candidate_to_candidate::*;
pub use client_to_node::*;
pub use error::*;
pub use follower_to_follower::*;
pub use follower_to_guest::*;
pub use follower_to_leader::*;
pub use guest_to_leader::*;
pub use guest_to_node::*;
pub use leader_to_follower::*;
pub use leader_to_guest::*;
pub use node_to_client::*;
pub use reader::*;
pub use writer::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    GuestToLeader(GuestToLeaderMessage),
    LeaderToGuest(LeaderToGuestMessage),

    FollowerToGuest(FollowerToGuestMessage),

    GuestToNode(GuestToNodeMessage),

    LeaderToFollower(LeaderToFollowerMessage),
    FollowerToLeader(FollowerToLeaderMessage),

    ClientToNode(ClientToNodeMessage),
    NodeToClient(NodeToClientMessage),

    CandidateToCandidate(CandidateToCandidateMessage),

    FollowerToFollower(FollowerToFollowerMessage),
}

impl From<Message> for Result<Message, MessageError> {
    fn from(message: Message) -> Self {
        Ok(message)
    }
}

impl From<GuestToLeaderMessage> for Message {
    fn from(message: GuestToLeaderMessage) -> Self {
        Self::GuestToLeader(message)
    }
}

impl From<LeaderToGuestMessage> for Message {
    fn from(message: LeaderToGuestMessage) -> Self {
        Self::LeaderToGuest(message)
    }
}

impl From<FollowerToGuestMessage> for Message {
    fn from(message: FollowerToGuestMessage) -> Self {
        Self::FollowerToGuest(message)
    }
}

impl From<GuestToNodeMessage> for Message {
    fn from(message: GuestToNodeMessage) -> Self {
        Self::GuestToNode(message)
    }
}

impl From<LeaderToFollowerMessage> for Message {
    fn from(message: LeaderToFollowerMessage) -> Self {
        Self::LeaderToFollower(message)
    }
}

impl From<FollowerToLeaderMessage> for Message {
    fn from(message: FollowerToLeaderMessage) -> Self {
        Self::FollowerToLeader(message)
    }
}

impl From<NodeToClientMessage> for Message {
    fn from(message: NodeToClientMessage) -> Self {
        Self::NodeToClient(message)
    }
}

impl From<ClientToNodeMessage> for Message {
    fn from(message: ClientToNodeMessage) -> Self {
        Self::ClientToNode(message)
    }
}

impl From<CandidateToCandidateMessage> for Message {
    fn from(message: CandidateToCandidateMessage) -> Self {
        Self::CandidateToCandidate(message)
    }
}

impl From<FollowerToFollowerMessage> for Message {
    fn from(message: FollowerToFollowerMessage) -> Self {
        Self::FollowerToFollower(message)
    }
}

impl Message {
    pub fn from_bytes(bytes: &[u8]) -> Result<(usize, Self), MessageError> {
        let (packet, remaining) = take_from_bytes::<Self>(bytes)?;
        Ok((bytes.len() - remaining.len(), packet))
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, MessageError> {
        Ok(to_allocvec(&self)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes, MessageError> {
        Ok(Bytes::from(self.to_vec()?))
    }
}

pub type MessageStreamReader = BasicMessageStreamReader<Message>;
pub type MessageStreamWriter = BasicMessageStreamWriter<Message>;
