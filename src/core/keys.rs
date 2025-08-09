//! TODO: Compress secret keys with raw bytes

use std::str::FromStr;

use anyhow::{Ok, anyhow, bail};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

impl SecretKeyV1 {
    /// Encode the secret key into a string in the form of:
    ///
    /// `aimo-sk-{scope}-{base58_encoded_secret_key_json}`
    pub fn into_string(self, scope: &str) -> anyhow::Result<String> {
        let raw = SecretKeyRawV1::try_from(self)?;
        let base58_encoded = bs58::encode(raw.into_bytes())
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

    pub fn decode(sk: &str) -> anyhow::Result<(String, Self)> {
        let (scope, key) = Self::split_sk_string(sk).ok_or(anyhow!(
            "Invalid secret key: Failed to split secret key into valid parts"
        ))?;
        let decoded_bytes = bs58::decode(key).into_vec()?;
        let raw = SecretKeyRawV1::from_bytes(&decoded_bytes[..])?;

        Ok((scope.to_string(), raw.try_into()?))
    }

    pub fn verify_signature(&self) -> anyhow::Result<()> {
        let metadata_bytes = MetadataRawV1::try_from(self.metadata.clone())?.into_bytes();
        let public_key = Pubkey::from_str(&self.signer)?;
        let signature = Signature::from_str(&self.signature)?;
        let is_valid = signature.verify(public_key.as_ref(), &metadata_bytes);

        if !is_valid {
            bail!("Wrong signature");
        }

        // Check expiry
        if let Some(dt) = DateTime::<Utc>::from_timestamp_millis(
            self.metadata.created_at + self.metadata.valid_for,
        ) {
            if dt < Utc::now() {
                bail!("Expired");
            }
        } else {
            bail!("Invalid timestamp");
        }

        Ok(())
    }

    pub fn into_hash(self) -> anyhow::Result<[u8; 32]> {
        let bytes = SecretKeyRawV1::try_from(self)?.into_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes[..]);
        Ok(hasher.finalize().into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetadataV1 {
    pub created_at: i64,
    pub valid_for: i64,
    pub usage_limit: u64,
    pub scopes: Vec<Scope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum Wallet {
    #[serde(rename = "solana")]
    Solana,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum Scope {
    #[serde(rename = "completion_model")]
    CompletionModel,
}

impl FromStr for Scope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "completion_model" => Ok(Self::CompletionModel),
            _ => Err(anyhow!("Scope {s} not supported")),
        }
    }
}

type WalletEnum = u8;
type ScopeBitMap = u64;

#[derive(Debug, Clone)]
pub struct SecretKeyRawV1 {
    pub version: u8,             // 1 byte
    pub wallet: WalletEnum,      // 1 byte
    pub signer: [u8; 32],        // 32 bytes
    pub signature: [u8; 64],     // 64 bytes
    pub metadata: MetadataRawV1, // 32 bytes
}

impl SecretKeyRawV1 {
    pub const BYTES: usize = 130; // 1 + 1 + 32 + 64 + 32

    pub fn into_bytes(self) -> Vec<u8> {
        [
            vec![self.version, self.wallet],
            self.signer.to_vec(),
            self.signature.to_vec(),
            self.metadata.into_bytes(),
        ]
        .concat()
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != Self::BYTES {
            bail!(
                "Bytes length doesn't match: expect {}, got {}",
                Self::BYTES,
                bytes.len()
            );
        }

        let version = bytes[0];
        let wallet = bytes[1];
        let signer: [u8; 32] = bytes[2..34].try_into()?;
        let signature: [u8; 64] = bytes[34..98].try_into()?;
        let metadata = MetadataRawV1::from_bytes(&bytes[98..130])?;

        Ok(Self {
            version,
            wallet,
            signer,
            signature,
            metadata,
        })
    }
}

impl TryFrom<SecretKeyV1> for SecretKeyRawV1 {
    type Error = anyhow::Error;

    fn try_from(value: SecretKeyV1) -> Result<Self, Self::Error> {
        if value.wallet != Wallet::Solana {
            bail!("Wallet not supported");
        }

        // Decode with base58 for solana wallets
        let signer: [u8; 32] = bs58::decode(&value.signer).into_vec()?[..].try_into()?;
        let signature: [u8; 64] = bs58::decode(&value.signature).into_vec()?[..].try_into()?;

        // TODO: Handle more wallets
        let wallet: WalletEnum = wallets::SOLANA;

        Ok(Self {
            version: value.version,
            wallet,
            signer,
            signature,
            metadata: value.metadata.try_into()?,
        })
    }
}

impl TryFrom<SecretKeyRawV1> for SecretKeyV1 {
    type Error = anyhow::Error;

    fn try_from(value: SecretKeyRawV1) -> Result<Self, Self::Error> {
        if value.wallet >= wallets::TOTAL_WALLETS_SUPPORTED {
            bail!("Unsupported wallet type in secret key: {}", value.wallet);
        }

        let wallet = match value.wallet {
            wallets::SOLANA => Wallet::Solana,

            // This will never happen
            _ => Wallet::Solana,
        };

        let signer = bs58::encode(&value.signer).into_string();
        let signature = bs58::encode(&value.signature).into_string();

        Ok(Self {
            version: value.version,
            wallet,
            signer,
            signature,
            metadata: value.metadata.try_into()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetadataRawV1 {
    pub created_at: i64,     // 8 bytes
    pub valid_for: i64,      // 8 bytes
    pub usage_limit: u64,    // 8 bytes
    pub scopes: ScopeBitMap, // 8 bytes
}

impl MetadataRawV1 {
    pub const BYTES: usize = 32; // 8 + 8 + 8 + 8

    pub fn into_bytes(self) -> Vec<u8> {
        // We want the bytes to be serialized "from left to right"
        // So we convert the numbers into big-endian bytes

        [
            self.created_at.to_be_bytes().to_vec(),
            self.valid_for.to_be_bytes().to_vec(),
            self.usage_limit.to_be_bytes().to_vec(),
            self.scopes.to_be_bytes().to_vec(),
        ]
        .concat()
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != Self::BYTES {
            bail!(
                "Bytes length doesn't match: expect {}, got {}",
                Self::BYTES,
                bytes.len()
            );
        }

        let created_at = i64::from_be_bytes(bytes[0..8].try_into()?);
        let valid_for = i64::from_be_bytes(bytes[8..16].try_into()?);
        let usage_limit = u64::from_be_bytes(bytes[16..24].try_into()?);
        let scopes = u64::from_be_bytes(bytes[24..32].try_into()?);

        Ok(Self {
            created_at,
            valid_for,
            usage_limit,
            scopes,
        })
    }
}

impl TryFrom<MetadataV1> for MetadataRawV1 {
    type Error = anyhow::Error;

    /// Keep `TryFrom` here even though this doesn't produce errors now
    fn try_from(value: MetadataV1) -> Result<Self, Self::Error> {
        // Convert options list into a bitmap
        let bitmap: ScopeBitMap = value.scopes.iter().fold(0, |bm, scope| match scope {
            Scope::CompletionModel => bm | 1 << scopes::COMPLETION_MODEL,
        });

        Ok(Self {
            created_at: value.created_at,
            valid_for: value.valid_for,
            usage_limit: value.usage_limit,
            scopes: bitmap,
        })
    }
}

impl TryFrom<MetadataRawV1> for MetadataV1 {
    type Error = anyhow::Error;

    fn try_from(value: MetadataRawV1) -> Result<Self, Self::Error> {
        // Scopes       Enabled
        // Supported    | 0     | 1
        //          0   | 1     | 0
        //          1   | 1     | 1
        // ------------------------
        // > is_valid = supported | !enabled
        // > is_invalid = !(supported | !enabled)
        //              = !supported & enabled
        if (!scopes::SCOPES_SUPPORTED & value.scopes) > 0 {
            bail!("Secret key contains currently unsupported scope type");
        }

        let scopes = if value.scopes | scopes::COMPLETION_MODEL > 0 {
            vec![Scope::CompletionModel]
        } else {
            vec![]
        };

        Ok(Self {
            created_at: value.created_at,
            valid_for: value.valid_for,
            usage_limit: value.usage_limit,
            scopes,
        })
    }
}

pub mod wallets {
    //! Signing wallets supported: `0x00` - `0xFF`

    use super::WalletEnum;
    pub const TOTAL_WALLETS_SUPPORTED: WalletEnum = 1;

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
    pub const COMPLETION_MODEL: ScopeBitMap = 0;
}
