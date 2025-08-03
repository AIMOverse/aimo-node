use axum::{Router, routing::get};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use super::state::GRpcState;

pub fn grpc_v1() -> Router {
    Router::new()
        .route("/ping", get(|| async { "gRPC service not available yet" }))
        .with_state(GRpcState::new())
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_grpc()))
}
