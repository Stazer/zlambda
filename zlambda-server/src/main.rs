mod application;
mod udp;
mod unix;
mod operation;
mod packet;
mod stdin;

////////////////////////////////////////////////////////////////////////////////////////////////////

use actix::System;
use application::ApplicationActor;
use std::error::Error;
use tokio::runtime::Builder;

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let system = System::with_tokio_rt(move || {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Cannot build tokio runtime")
    });

    system.block_on(async {
        let application = ApplicationActor::new();
        application
            .send(application::CreateUdpSocketMessage::new("0.0.0.0:8080"))
            .await;

        stdin::StdinActor::new();
    });

    system.run()?;

    Ok(())
}
