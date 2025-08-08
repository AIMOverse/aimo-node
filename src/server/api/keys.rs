use axum::{Json, http::StatusCode};

use crate::{
    core::keys::{MetadataRawV1, SecretKeyV1},
    server::types::keys::{
        GenerateKeyRequest, GenerateKeyResponse, MetadataBytesRequest, VerifyKeyRequest,
        VerifyKeyResponse,
    },
};

/// GET /keys/metadata_bytes
pub async fn metadata_bytes(
    Json(body): Json<MetadataBytesRequest>,
) -> Result<Json<Vec<u8>>, (StatusCode, String)> {
    let metadata = body.metadata;
    let bytes = MetadataRawV1::try_from(metadata)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
        .into_bytes();

    Ok(Json(bytes))
}

/// POST /keys/generate
pub async fn generate_key(
    Json(body): Json<GenerateKeyRequest>,
) -> Result<Json<GenerateKeyResponse>, (StatusCode, String)> {
    let sk_encoded = body
        .payload
        .into_string("dev")
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    Ok(Json(GenerateKeyResponse {
        secret_key: sk_encoded,
    }))
}

/// POST /keys/verify
pub async fn verify_key(
    Json(body): Json<VerifyKeyRequest>,
) -> Result<Json<VerifyKeyResponse>, (StatusCode, String)> {
    let (scope, payload) = SecretKeyV1::decode(&body.secret_key)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    if scope != "dev" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Scope {scope} not supported"),
        ));
    }

    let result = payload.verify_signature();

    Ok(Json(VerifyKeyResponse {
        result: result.is_ok(),
        reason: result.map_or_else(|err| Some(err.to_string()), |_| None),
        payload,
    }))
}
