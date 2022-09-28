#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Parser, Subcommand};
use std::error::Error;
use zlambda_server::cluster::NodeActor;
use zlambda_server::run::run_with_default_system_runner;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct MainArguments {
    #[clap(subcommand)]
    command: MainCommand,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum MainCommand {
    Server {
        #[clap(default_value = "0.0.0.0:8000")]
        listener_address: String,
        leader_address: Option<String>,
    },
    Client,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = MainArguments::parse();

    match arguments.command {
        MainCommand::Server {
            listener_address,
            leader_address,
        } => {
            run_with_default_system_runner(async move || {
                NodeActor::new(listener_address, leader_address).await?;
                Ok(())
            })
            .expect("Cannot start manager");
        }
        MainCommand::Client => {}
    };

    Ok(())
}
