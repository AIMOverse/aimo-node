use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::Value;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use crate::router::Request;
use crate::server::api::state::ApiState;
use crate::types::keys::SecretKeyV1;

/// Expose an openai-compatible API
///
/// POST /chat/completions
// #[axum::debug_handler]
pub async fn completions(
    Extension(payload): Extension<SecretKeyV1>,
    State(ApiState { ctx }): State<ApiState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let target = body
        .get("target")
        .ok_or((
            StatusCode::BAD_REQUEST,
            "`target` field not specified".to_string(),
        ))?
        .to_string();

    let mut rx = ctx
        .router
        .route_request(Request {
            service_id: target,
            sender_id: payload.signer.clone(),
            request_id: Keypair::new().pubkey().to_string(),
            endpoint: None,
            request_type: "completion_model".to_string(),
            payload: body.to_string(),
            headers: HashMap::new(),
            payload_encrypted: false,
            signature: None,
        })
        .await
        .map_err(|err| {
            (
                StatusCode::NOT_FOUND,
                format!("Failed to route request: {err}"),
            )
        })?;

    let response = rx.recv().await.ok_or((
        StatusCode::NOT_FOUND,
        format!("Failed to receive responses"),
    ))?;

    Ok(Json(serde_json::to_value(response).map_err(|err| {
        (
            StatusCode::NOT_FOUND,
            format!("Invalid response received: {err}"),
        )
    })?))
}
