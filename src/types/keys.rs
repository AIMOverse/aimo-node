//! TODO: Compress secret keys with raw bytes

use std::str::FromStr;

use anyhow::anyhow;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

/// Secret key format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretKeyV1 {
    pub version: u8,          // 1 byte
    pub wallet: Wallet,       // use enum: 1 byte
    pub signer: String,       // 32 bytes
    pub signature: String,    // 64 bytes
    pub metadata: MetadataV1, // 28 bytes
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetadataV1 {
    pub created_at: u64,    // 8 bytes
    pub valid_for: u64,     // 8 bytes
    pub usage_limit: u64,   // 8 bytes
    pub scopes: Vec<Scope>, // use bitmap: 4 bytes
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum Wallet {
    #[serde(rename = "solana")]
    Solana,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum Scope {
    #[serde(rename = "model:completion")]
    ModelCompletion,
}

type WalletEnum = u8;
type ScopeBitMap = u32;

pub mod wallets {
    //! Signing wallets supported: `0x00` - `0xFF`

    use super::WalletEnum;
    pub const TOTAL_WALLETS_SUPPORTED: usize = 1;

    /// The default option: Solana wallets
    pub const SOLANA: WalletEnum = 0x00;
}

pub mod scopes {
    //! Scope bitmap options: 0 - 31, from lower bit to higher bit
    //!

    use super::ScopeBitMap;

    pub const SCOPES_SUPPORTED: ScopeBitMap = 0x01;

    /// Scope: `model:completion`
    ///
    /// Position: `0x01` (1 << 0)
    pub const CompletionModel: usize = 0;
}

// impl MetadataV1 {
//     pub fn into_bytes(self) -> Vec<u8> {
//         // We want the bytes to be serialized "from left to right"
//         // So we convert the numbers into big-endian bytes

//         [
//             self.created_at.to_be_bytes().to_vec(),
//             self.valid_for.to_be_bytes().to_vec(),
//             self.usage_limit.to_be_bytes().to_vec(),
//             self.scopes.to_be_bytes().to_vec(),
//         ]
//         .concat()
//     }

//     pub fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {}
// }

impl SecretKeyV1 {
    /// Encode the secret key into a string in the form of:
    ///
    /// `aimo-sk-{scope}-{base58_encoded_secret_key_json}`
    pub fn try_encode(&self, scope: &str) -> anyhow::Result<String> {
        let sk_json = serde_json::to_string(self)?;
        let base58_encoded = bs58::encode(sk_json.into_bytes())
            // .with_check()
            .into_string();

        Ok(format!("aimo-sk-{scope}-{base58_encoded}"))
    }

    fn split_sk_string(sk: &str) -> Option<(&str, &str)> {
        let mut parts = sk.splitn(4, '-');
        let aimo = parts.next()?;
        if aimo != "aimo" {
            return None;
        }
        let prefix = parts.next()?;
        if prefix != "sk" {
            return None;
        }

        let scope = parts.next()?;
        let base58_value = parts.next()?;

        Some((scope, base58_value))
    }

    pub fn try_decode(sk: &str) -> anyhow::Result<(String, Self)> {
        let (scope, key) = Self::split_sk_string(sk).ok_or(anyhow!(
            "Invalid secret key: Failed to split secret key into valid parts"
        ))?;
        let decoded = bs58::decode(key).into_vec()?;
        let key_string = String::from_utf8(decoded)?;

        Ok((
            scope.to_string(),
            serde_json::from_str::<Self>(&key_string)
                .map_err(|err| anyhow!("Failed to deserialize secret key: {err}"))?,
        ))
    }

    pub fn verify_signature(&self) -> anyhow::Result<bool> {
        let metadata_bytes = serde_json::to_string(&self.metadata)?.into_bytes();
        let public_key = Pubkey::from_str(&self.signer)?;
        let signature = Signature::from_str(&self.signature)?;
        let is_valid = signature.verify(public_key.as_ref(), &metadata_bytes);
        Ok(is_valid)
    }

    // pub fn into_bytes(self) -> Vec<u8> {
    //     [
    //         vec![self.version, self.wallet],
    //         self.signer.to_vec(),
    //         self.signature.to_vec(),
    //         self.metadata.into_bytes(),
    //     ]
    //     .concat()
    // }
}
