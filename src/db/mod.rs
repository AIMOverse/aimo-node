use std::path::{Path, PathBuf};

pub struct StateDb {
    keys_db: sled::Db,
}

pub const KEYS_DB_NAME: &'static str = "keys.db";

impl StateDb {
    pub fn load_or_create(directory: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            keys_db: sled::open(directory.join(KEYS_DB_NAME))?,
        })
    }
}

pub fn default_directory() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or(PathBuf::from("."))
        .join("aimo")
        .join("state")
}
