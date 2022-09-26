#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Parser, Subcommand};
use std::error::Error;
use zlambda_server::cluster::manager::ManagerActor;
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
        #[clap(subcommand)]
        command: ServerCommand,
    },
    Client,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum ServerCommand {
    Manager {
        #[clap(default_value = "0.0.0.0:8000")]
        listener_address: String,
        leader_address: Option<String>,
    },
    Worker {},
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = MainArguments::parse();

    match arguments.command {
        MainCommand::Server { command } => {
            match command {
                ServerCommand::Manager {
                    listener_address,
                    leader_address,
                } => {
                    run_with_default_system_runner(async move || {
                        ManagerActor::new(listener_address, leader_address).await?;
                        Ok(())
                    })
                    .expect("Cannot start manager");
                }
                ServerCommand::Worker {} => {}
            };
        }
        MainCommand::Client => {}
    };

    Ok(())
}
