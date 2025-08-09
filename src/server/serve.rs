use std::sync::Arc;

use axum::Router;

use crate::{
    config::ServerOptions,
    db::StateDb,
    server::{api::api_v1, context::ServiceContext, grpc::grpc_v1},
};

pub async fn serve(options: &ServerOptions, ctx: ServiceContext, state_db: Arc<StateDb>) {
    let router = Router::new()
        // Setup router groups
        .nest("/api/v1", api_v1(options, ctx, state_db))
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
