use axum::{Json, http::StatusCode};

use crate::{server::types::keys::GenerateKeyResponse, types::keys::SecretKeyV1};

/// POST /api/v1/keys/generate
pub async fn generate_key(
    Json(payload): Json<SecretKeyV1>,
) -> Result<Json<GenerateKeyResponse>, (StatusCode, String)> {
    let sk_encoded = payload
        .try_encode("dev")
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    Ok(Json(GenerateKeyResponse {
        secret_key: sk_encoded,
    }))
}
