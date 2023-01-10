use tokio::{select, spawn};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerFollowerTask {

}


impl FollowerFollowerTask {
    pub fn new() -> Self {
        Self {}
    }

    pub fn spawn(self) {
        spawn(async move {

        });
    }

    pub fn select(&mut self) {

    }
}
