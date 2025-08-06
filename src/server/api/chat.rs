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
    let mut body_cloned = body.clone();
    let mut model = body
        .get("model")
        .ok_or((
            StatusCode::BAD_REQUEST,
            "`model` field not specified".to_string(),
        ))?
        .as_str()
        .ok_or((
            StatusCode::BAD_REQUEST,
            "`model` field must be a string".to_string(),
        ))?
        .splitn(2, ':');

    let target = model.next().ok_or((
        StatusCode::BAD_REQUEST,
        "Can't parse target: Invalid `model` field: Should be in this pattern: \"<target>:<model_name>\"".to_string(),
    ))?;

    let model_name = model.next().ok_or((
        StatusCode::BAD_REQUEST,
        "Can't parse model name: Invalid `model` field: Should be in this pattern: \"<target>:<model_name>\"".to_string(),
    ))?;

    body_cloned["model"] = Value::String(model_name.to_string());

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    let mut rx = ctx
        .router
        .route_request(Request {
            service_id: target.to_string(),
            sender_id: payload.signer.clone(),
            request_id: Keypair::new().pubkey().to_string(),
            endpoint: None,
            request_type: "completion_model".to_string(),
            method: "POST".to_string(),
            payload: body_cloned.to_string(),
            headers,
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

    let status_code =
        StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body =
        serde_json::from_str::<Value>(&response.payload).unwrap_or(Value::String(response.payload));

    if !status_code.is_success() {
        return Err((status_code, body.to_string()));
    }

    Ok(Json(body))
}
