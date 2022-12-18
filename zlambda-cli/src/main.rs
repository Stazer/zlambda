#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::io::{stdin, stdout};
use zlambda_client::Client;
use zlambda_common::module::ModuleId;
use zlambda_common::node::NodeId;
use zlambda_server::Server;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct MainArguments {
    #[clap(subcommand)]
    command: MainCommand,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Args)]
struct FollowerData {
    address: String,
    node_id: NodeId,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum MainCommand {
    Server {
        #[clap(default_value = "0.0.0.0:8000")]
        listener_address: String,
        #[clap(subcommand)]
        command: ServerCommand,
    },
    Client {
        #[clap(default_value = "0.0.0.0:8000")]
        address: String,
        #[clap(subcommand)]
        command: ClientCommand,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum ServerCommand {
    Leader,
    Follower {
        leader_address: String,
        node_id: Option<NodeId>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum ClientCommand {
    Load { path: PathBuf },
    Dispatch { id: ModuleId },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let arguments = MainArguments::parse();

    match arguments.command {
        MainCommand::Server {
            listener_address,
            command,
        } => {
            tracing_subscriber::fmt::init();

            let server = match command {
                ServerCommand::Leader => Server::new::<_, String>(listener_address, None).await?,
                ServerCommand::Follower {
                    leader_address,
                    node_id,
                } => Server::new(listener_address, Some((leader_address, node_id))).await?,
            };

            server.run().await;
        }
        MainCommand::Client { address, command } => {
            let mut client = match Client::new(address).await {
                Err(error) => return Err(error),
                Ok(client) => client,
            };

            match command {
                ClientCommand::Load { path } => {
                    let id = client.load_module(&path).await?;

                    println!("{}", id);
                }
                ClientCommand::Dispatch { id } => {
                    stdout()
                        .write_all(&client.dispatch(id, stdin()).await?)
                        .await?;
                }
            };
        }
    };

    Ok(())
}
