use axum::{Router, routing::get};

use super::state::ApiState;

pub fn api_v1() -> Router {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .with_state(ApiState::new())
}
