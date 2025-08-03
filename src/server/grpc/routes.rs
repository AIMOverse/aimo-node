use axum::{Router, routing::get};

use super::state::GRpcState;

pub fn grpc_v1() -> Router {
    Router::new()
        .route("/ping", get(|| async { "gRPC service not available yet" }))
        .with_state(GRpcState::new())
}
