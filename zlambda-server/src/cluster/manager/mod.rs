pub mod follower;
pub mod leader;
use actix::System;

////////////////////////////////////////////////////////////////////////////////////////////////////

use std::error::Error;
use tokio::runtime::Builder;

pub fn main() -> Result<(), Box<dyn Error>> {
    let system = System::with_tokio_rt(move || {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Cannot build tokio runtime")
    });

    system.block_on(async {
        leader::ManagerLeaderActor::new("0.0.0.0:8000")
            .await
            .expect("Error binding socket");
    });

    system.run()?;

    Ok(())
}

pub fn main2(leader_address: &str) -> Result<(), Box<dyn Error>> {
    let system = System::with_tokio_rt(move || {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Cannot build tokio runtime")
    });

    system.block_on(async {
        follower::ManagerFollowerActor::new("0.0.0.0:8001", leader_address)
            .await
            .expect("Error binding socket or connecting");
    });

    system.run()?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ManagerFollowerState {
    Handshaking,
    Operational,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ManagerId = usize;
