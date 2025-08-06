use std::path::PathBuf;

use chrono::{Duration, Utc};
use solana_sdk::signer::Signer;

use crate::{
    types::keys::{MetadataRawV1, MetadataV1, Scope, SecretKeyV1, Wallet},
    utils::id::create_keypair_from_file,
};

/// Generate a secret key with given metadata and keypair.
///
/// NOTE: `valid_for` is how many **DAYS** this key should be valid for.
pub fn generate_secret_key(
    tag: &str,
    valid_for: u32,
    scopes: Vec<Scope>,
    usage_limit: u64,
    id: Option<PathBuf>,
) -> anyhow::Result<String> {
    let keypair = create_keypair_from_file(id)?;
    let valid_for = Duration::days(valid_for.into()).num_milliseconds();
    let created_at = Utc::now().timestamp_millis();

    let metadata = MetadataV1 {
        created_at,
        usage_limit,
        valid_for,
        scopes,
    };

    let bytes = MetadataRawV1::try_from(metadata.clone())?.into_bytes();
    let signature = keypair.sign_message(&bytes[..]).to_string();
    let signer = keypair.pubkey().to_string();

    let payload = SecretKeyV1 {
        version: 1,
        wallet: Wallet::Solana,
        signer,
        signature,
        metadata,
    };

    Ok(payload.into_string(tag)?)
}
