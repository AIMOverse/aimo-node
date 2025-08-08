use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Response, Sse};
use axum::{Extension, Json};
use futures_util::stream;
use serde_json::Value;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use crate::core::{keys::SecretKeyV1, transport::Request};
use crate::server::api::state::ApiState;

/// Expose an openai-compatible API
///
/// POST /chat/completions
// #[axum::debug_handler]
pub async fn completions(
    Extension(payload): Extension<SecretKeyV1>,
    State(ApiState { ctx }): State<ApiState>,
    Json(body): Json<Value>,
) -> Result<Response, (StatusCode, String)> {
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

    // Check if the response is a streaming response
    let is_stream = response
        .headers
        .get("content-type")
        .map(|ct| ct.contains("text/event-stream"))
        .unwrap_or(false);

    if is_stream {
        // Handle streaming response
        let first_chunk = response.payload.clone();
        let stream = stream::unfold(
            (Some(first_chunk), Some(rx)),
            move |(chunk_opt, mut rx_opt)| async move {
                if let Some(chunk) = chunk_opt {
                    // Return the first chunk
                    if !chunk.is_empty() {
                        let event = Event::default().data(chunk);
                        return Some((Ok::<Event, axum::Error>(event), (None, rx_opt)));
                    }
                }

                // Continue receiving chunks
                if let Some(ref mut rx) = rx_opt {
                    if let Some(next_response) = rx.recv().await {
                        if !next_response.payload.is_empty() {
                            let event = Event::default().data(next_response.payload);
                            return Some((Ok::<Event, axum::Error>(event), (None, rx_opt)));
                        }
                    }
                }
                None
            },
        );

        let sse = Sse::new(stream).keep_alive(KeepAlive::default());
        Ok(sse.into_response())
    } else {
        // Handle regular JSON response
        let body = serde_json::from_str::<Value>(&response.payload)
            .unwrap_or(Value::String(response.payload));

        if !status_code.is_success() {
            return Err((status_code, body.to_string()));
        }

        Ok(Json(body).into_response())
    }
}
