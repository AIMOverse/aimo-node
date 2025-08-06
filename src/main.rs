use std::{process, sync::Arc};

use clap::Parser;
use tokio::task::JoinSet;

use crate::{
    cli::{CliArgs, CommandArgs},
    config::ServerOptions,
    router::local::LocalRouter,
    serve::run_serve,
    server::ServiceContext,
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
