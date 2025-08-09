use std::str::FromStr;

use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::{
    core::{
        keys::{MetadataRawV1, SecretKeyV1},
        state::events::KeyRevocation,
    },
    server::{
        api::state::ApiState,
        types::keys::{
            GenerateKeyRequest, GenerateKeyResponse, MetadataBytesRequest, RevokeKeyRequest,
            VerifyKeyRequest, VerifyKeyResponse,
        },
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

/// POST /keys/revoke
#[axum::debug_handler]
pub async fn revoke_key(
    // Extension(payload): Extension<SecretKeyV1>,
    State(ApiState { state_db, .. }): State<ApiState>,
    Json(body): Json<RevokeKeyRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let signer = Pubkey::from_str(&body.signer)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid signer".to_string()))?;
    let signature = Signature::from_str(&body.signature).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid signature format".to_string(),
        )
    })?;
    if !signature.verify(&signer.to_bytes(), body.secret_key.as_bytes()) {
        return Err((StatusCode::UNAUTHORIZED, "Wrong signature".to_string()));
    }

    let (_, payload) = SecretKeyV1::decode(&body.secret_key).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid secret key: {err}"),
        )
    })?;

    if body.signer != payload.signer {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Request signer is different from secret key signer".to_string(),
        ));
    }

    let event = KeyRevocation {
        key: body.secret_key,
    };

    state_db
        .revocation
        .revoke_key(event)
        .map(|_| Json(json!({})))
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to revoke key internally: {err}"),
            )
        })
}
