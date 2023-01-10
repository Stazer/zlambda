use crate::follower::follower::FollowerFollowerTask;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowerFollowerBuilder {
}

impl FollowerFollowerBuilder {
    pub fn task(self) -> FollowerFollowerTask {
        FollowerFollowerTask::new()
    }
}
