mod api;

pub use api::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_options: ApiOptions,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            api_options: ApiOptions::from_env(),
        }
    }
}
