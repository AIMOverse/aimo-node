use std::path::PathBuf;

use anyhow::anyhow;
use solana_sdk::signature::{Keypair, read_keypair_file};

/// Create a solana keypair from `id.json` file.
///
/// If `path` is `None`, use the default path `~/.config/solana/id.json`
///
/// Returns error if we can't locate home directory or can't read keypair file properly
pub fn create_keypair_from_file(path: Option<PathBuf>) -> anyhow::Result<Keypair> {
    let home_dir = dirs::home_dir().ok_or(anyhow!("Can't locate home directory"))?;
    let default_path = [".config", "solana", "id.json"];

    let path = path.unwrap_or_else(|| {
        default_path.iter().fold(home_dir, |mut path, p| {
            path.push(p);
            path
        })
    });

    read_keypair_file(path).map_err(|err| anyhow!("Failed to read keypair file: {err}"))
}
