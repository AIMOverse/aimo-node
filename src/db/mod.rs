use std::path::{Path, PathBuf};

use anyhow::Ok;
use chrono::Utc;

use crate::core::{keys::SecretKeyV1, state::events};

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

impl StateDb {
    pub fn revoke_key(&self, event: events::KeyRevocation) -> anyhow::Result<()> {
        let events::KeyRevocation { key } = event;
        let (_, secret_key) = SecretKeyV1::decode(&key)?;
        let hash = secret_key.into_hash()?;
        let now = Utc::now().timestamp_millis();

        if self.keys_db.get(hash)?.is_none() {
            self.keys_db.insert(hash, &now.to_be_bytes())?;
        }

        tracing::debug!("Revoked key {key}");

        Ok(())
    }

    pub fn is_key_revoked(&self, key: &SecretKeyV1) -> anyhow::Result<bool> {
        Ok(self.keys_db.get(key.clone().into_hash()?)?.is_some())
    }
}
