use axum::{
    Router, middleware,
    routing::{any, get, post},
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    config::ServerOptions,
    server::{
        api::{
            chat::completions,
            keys::{generate_key, metadata_bytes, verify_key},
            subscribe,
        },
        context::ServiceContext,
        middleware::{auth_layer, cors_layer, timeout_layer},
    },
};

use super::state::ApiState;

pub fn api_v1(options: &ServerOptions, ctx: ServiceContext) -> Router {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/keys/metadata_bytes", get(metadata_bytes))
        .route("/keys/generate", post(generate_key))
        .route("/keys/verify", post(verify_key))
        .route(
            "/chat/completions",
            post(completions).layer(middleware::from_fn(auth_layer)),
        )
        .route(
            "/providers/subscribe",
            any(subscribe::handler).layer(middleware::from_fn(auth_layer)),
        )
        .with_state(ApiState::new(ctx))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer(&options))
                .layer(timeout_layer(&options)),
        )
}
