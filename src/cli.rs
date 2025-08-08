use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::core::keys::Scope;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CommandArgs,
}

#[derive(Debug, Subcommand)]
pub enum CommandArgs {
    /// Run AiMo Network node service
    Serve {
        /// The port the server listens on
        #[arg(long, short, default_value_t = 8000)]
        port: u16,

        /// The host address the server runs on
        #[arg(long, short, default_value = "0.0.0.0")]
        addr: String,

        /// Path to the node's Solana wallet id file
        #[arg(
            long,
            value_name = "FILE",
            long_help = "Specify a Solana wallet id file (id.json, generated with `solana-keygen new`) the node will be using. Defaults to `~/.config/solana/id.json`."
        )]
        id: Option<PathBuf>,
    },

    /// Generate a secret key for your wallet
    Keygen {
        /// The scope tag of the secret key, e.g. dev
        #[arg(
            long,
            short,
            default_value = "dev",
            long_help = "Scope tag is set to `dev` by default. By specifying this, you get a secret key like: `aimo-sk-<tag>-xxxxxxx`"
        )]
        tag: String,

        /// How many days should the secret key valid for
        #[arg(long, short, default_value_t = 90)]
        valid_for: u32,

        /// Scopes to enable for this secret key
        #[arg(
            long,
            short,
            value_delimiter = ',',
            long_help = "Specify which scopes to enable with comma-seperated values. Current supported values are: \"completion_model\"",
            default_value = "completion_model"
        )]
        scopes: Vec<Scope>,

        /// Usage limit of the secret key
        #[arg(long, short, default_value_t = 0)]
        usage_limit: u64,

        /// Path to secret key signer's Solana wallet id file
        #[arg(
            long,
            value_name = "FILE",
            long_help = "Specify a Solana wallet id file (id.json, generated with `solana-keygen new`) to sign the secret key. Defaults to `~/.config/solana/id.json`."
        )]
        id: Option<PathBuf>,
    },

    /// Run a proxy to connect your endpoint to AiMo Network directly
    Proxy {
        /// Url to an AiMo Network node
        #[arg(long)]
        node_url: String,

        /// AiMo Network secret key
        #[arg(
            long,
            long_help = "Provide your secret key to access AiMo Network nodes. See `aimo keygen --help`."
        )]
        secret_key: String,

        /// Url to your service endpoint
        #[arg(long)]
        endpoint_url: String,

        /// If your service endpoint requires an API key, specify the key here.
        #[arg(long)]
        api_key: Option<String>,
    },
}
