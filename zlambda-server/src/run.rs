use actix::{System, SystemRunner};
use std::error::Error;
use std::future::Future;
use tokio::runtime::Builder;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn default_system_runner() -> SystemRunner {
    System::with_tokio_rt(move || {
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Cannot builder runtime")
    })
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn run_with_default_system_runner<T, F>(function: T) -> Result<(), Box<dyn Error>>
where
    T: FnOnce() -> F,
    F: Future<Output = Result<(), Box<dyn Error>>>,
{
    let system = default_system_runner();

    system.block_on(async {
        function().await.unwrap();
    });

    system.run()?;

    Ok(())
}
