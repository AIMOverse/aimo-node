use clap::Parser;

use crate::{
    cli::{CliArgs, CommandArgs},
    serve::run_serve,
};

mod cli;
mod config;
mod helpers;
mod router;
mod serve;
mod server;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = CliArgs::parse();

    match args.command {
        CommandArgs::Serve { port, addr, id } => {
            run_serve(addr, port, id).await;
        }
        CommandArgs::Keygen {
            valid_for,
            scopes,
            usage_limit,
            id,
        } => {}
    }
}
