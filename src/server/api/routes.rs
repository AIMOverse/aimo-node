use axum::{Router, routing::get};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    config::ServerOptions,
    server::{
        context::ServiceContext,
        middleware::{cors_layer, timeout_layer},
    },
};

use super::state::ApiState;

pub fn api_v1(options: &ServerOptions, ctx: ServiceContext) -> Router {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .with_state(ApiState::new(ctx))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer(&options))
                .layer(timeout_layer(&options)),
        )
}
