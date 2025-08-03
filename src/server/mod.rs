use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    config::ApiOptions,
    server::{
        api::api_v1,
        grpc::grpc_v1,
        middleware::{cors_layer, timeout_layer},
    },
};

mod api;
mod grpc;
mod middleware;

pub async fn serve(options: &ApiOptions) {
    let router = Router::new()
        // Setup router groups
        .nest("/api/v1", api_v1())
        .nest("/grpc/v1", grpc_v1())
        // Setup middlewares
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TraceLayer::new_for_grpc())
                .layer(cors_layer(&options))
                .layer(timeout_layer(&options)),
        );

    tracing::info!("Server instance built");

    let socket_addr = format!("{}:{}", options.addr, options.port);
    // run our app with hyper, listening globally on configured address and port
    let listener = tokio::net::TcpListener::bind(&socket_addr)
        .await
        .inspect_err(|err| tracing::error!("{err}"))
        .unwrap();

    tracing::info!("API server running and listening at {socket_addr}");

    axum::serve(listener, router)
        .await
        .inspect_err(|err| tracing::error!("{err}"))
        .unwrap();
}
