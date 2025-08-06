use std::process;

use clap::Parser;

use crate::{
    cli::{CliArgs, CommandArgs},
    helpers::{keygen::generate_secret_key, proxy},
    node::run_serve,
};

mod cli;
mod config;
mod helpers;
mod node;
mod router;
mod server;
mod types;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = CliArgs::parse();

    match args.command {
        // aimo serve
        CommandArgs::Serve { port, addr, id } => {
            run_serve(addr, port, id).await;
        }

        // aimo keygen
        CommandArgs::Keygen {
            tag,
            valid_for,
            scopes,
            usage_limit,
            id,
        } => {
            if let Err(err) = generate_secret_key(&tag, valid_for, scopes, usage_limit, id)
                .map(|sk| println!("{sk}"))
            {
                println!("Error: {err}");
                process::exit(1);
            }
        }

        // aimo proxy
        CommandArgs::Proxy {
            node_url,
            secret_key,
            endpoint_url,
            api_key,
        } => {
            if let Err(err) =
                proxy::serve_websocket(node_url, secret_key, endpoint_url, api_key).await
            {
                println!("Error: {err}");
                process::exit(1);
            }
        }
    }
}
