use anyhow::Ok;
use chrono::Utc;

use crate::core::{keys::SecretKeyV1, state::events};

pub struct RevocationDb(pub sled::Db);

impl RevocationDb {
    pub fn revoke_key(&self, event: events::KeyRevocation) -> anyhow::Result<()> {
        let events::KeyRevocation { key } = event;
        let (_, secret_key) = SecretKeyV1::decode(&key)?;
        let hash = secret_key.into_hash()?;
        let now = Utc::now().timestamp_millis();

        if self.0.get(hash)?.is_none() {
            self.0.insert(hash, &now.to_be_bytes())?;
        }

        tracing::debug!("Revoked key {key}");

        Ok(())
    }

    pub fn is_key_revoked(&self, key: &SecretKeyV1) -> anyhow::Result<bool> {
        Ok(self.0.get(key.clone().into_hash()?)?.is_some())
    }
}
