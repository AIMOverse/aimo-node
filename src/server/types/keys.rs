use serde::{Deserialize, Serialize};

use crate::types::keys::{MetadataV1, SecretKeyV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataBytesRequest {
    pub metadata: MetadataV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateKeyRequest {
    pub payload: SecretKeyV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateKeyResponse {
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyKeyRequest {
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyKeyResponse {
    pub result: bool,
    pub reason: Option<String>,
    pub payload: SecretKeyV1,
}
