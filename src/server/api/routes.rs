use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{any, get, post},
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    config::ServerOptions,
    db::StateDb,
    server::{
        api::{
            chat::completions,
            keys::{generate_key, metadata_bytes, revoke_key, verify_key},
            subscribe,
        },
        context::ServiceContext,
        middleware::{auth_layer, cors_layer, timeout_layer},
    },
};

use super::state::ApiState;

pub fn api_v1(options: &ServerOptions, ctx: ServiceContext, state_db: Arc<StateDb>) -> Router {
    let state = ApiState::new(ctx, state_db);
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/keys/metadata_bytes", get(metadata_bytes))
        .route("/keys/generate", post(generate_key))
        .route("/keys/verify", post(verify_key))
        .route("/keys/revoke", post(revoke_key))
        .route(
            "/chat/completions",
            post(completions).layer(middleware::from_fn_with_state(state.clone(), auth_layer)),
        )
        .route(
            "/providers/subscribe",
            any(subscribe::handler)
                .layer(middleware::from_fn_with_state(state.clone(), auth_layer)),
        )
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors_layer(&options))
                .layer(timeout_layer(&options)),
        )
}
