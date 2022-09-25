use clap::{Parser, Subcommand};
use std::error::Error;

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
    Manager { leader_address: Option<String> },
    Worker {},
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = MainArguments::parse();

    match arguments.command {
        MainCommand::Server { command } => {
            match command {
                ServerCommand::Manager { leader_address } => match leader_address {
                    Some(leader_address) => {
                        zlambda_server::cluster::manager::main2(&leader_address)?;
                    }
                    None => {
                        zlambda_server::cluster::manager::main()?;
                    }
                },
                ServerCommand::Worker {} => {}
            };
        }
        MainCommand::Client => {}
    };

    Ok(())
}
