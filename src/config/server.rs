use std::env;

use serde::{Deserialize, Serialize};

use crate::cli::CommandArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerOptions {
    pub addr: String,
    pub port: u16,
}

impl Default for ServerOptions {
    /// Server runs at `0.0.0.0:8000` by default
    fn default() -> Self {
        ServerOptions {
            addr: String::from("0.0.0.0"),
            port: 8000,
        }
    }
}

impl ServerOptions {
    pub fn from_env() -> Self {
        let mut default_opts = Self::default();

        if let Ok(addr) = env::var("AIMO_LISTEN_ADDRESS") {
            default_opts.addr = addr;
        }

        if let Ok(port_str) = env::var("AIMO_LISTEN_PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                default_opts.port = port;
            }
        }

        default_opts
    }
}
