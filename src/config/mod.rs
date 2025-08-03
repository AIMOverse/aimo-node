mod server;

use serde::{Deserialize, Serialize};
pub use server::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_options: ServerOptions,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            server_options: ServerOptions::from_env(),
        }
    }
}
