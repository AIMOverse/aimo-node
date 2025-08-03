use axum::Router;

use crate::{
    config::ServerOptions,
    server::{api::api_v1, grpc::grpc_v1},
};

mod api;
mod grpc;
mod middleware;

pub async fn serve(options: &ServerOptions) {
    let router = Router::new()
        // Setup router groups
        .nest("/api/v1", api_v1(options))
        .nest("/grpc/v1", grpc_v1());

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
