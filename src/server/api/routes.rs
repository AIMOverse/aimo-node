use axum::{Router, routing::get};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    config::ServerOptions,
    server::middleware::{cors_layer, timeout_layer},
};

use super::state::ApiState;

pub fn api_v1(options: &ServerOptions) -> Router {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .with_state(ApiState::new())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer(&options))
                .layer(timeout_layer(&options)),
        )
}
