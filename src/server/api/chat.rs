use axum::Json;
use axum::http::StatusCode;
use serde_json::{Value, json};

/// Expose an openai-compatible API
///
/// POST /chat/completions
pub async fn completions(Json(payload): Json<Value>) -> Result<Json<Value>, (StatusCode, String)> {
    if let Some(Value::Bool(is_stream)) = payload.get("stream") {}

    Ok(Json(json!({})))
}
