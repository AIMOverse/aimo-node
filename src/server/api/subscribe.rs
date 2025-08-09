use axum::{
    Extension,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::task::JoinSet;

use crate::{
    core::{keys::SecretKeyV1, transport},
    server::{ServiceContext, api::state::ApiState},
};

pub async fn handler(
    Extension(payload): Extension<SecretKeyV1>,
    ws: WebSocketUpgrade,
    State(ApiState { ctx, .. }): State<ApiState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, ctx, payload))
}

async fn handle_socket(mut socket: WebSocket, ctx: ServiceContext, payload: SecretKeyV1) {
    match ctx.router.register_service(payload.signer.clone()).await {
        Ok(connection) => {
            let (mut ws_sender, mut ws_receiver) = socket.split();
            let tx = connection.tx.clone();
            let mut rx = connection.rx;
            let mut js = JoinSet::new();

            // Forward requests to service provider
            js.spawn(async move {
                while let Some(request) = rx.recv().await {
                    if let Ok(msg) = serde_json::to_string(&request) {
                        if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                            tracing::warn!("Service provider disconnected");
                            break;
                        }
                    }
                }
            });

            // Client to router
            js.spawn(async move {
                while let Some(Ok(Message::Text(text))) = ws_receiver.next().await {
                    let str = text.to_string();
                    if let Ok(response) = serde_json::from_str::<transport::Response>(&str)
                        .inspect_err(|err| tracing::debug!("Failed to deserialize response: {err}"))
                    {
                        if let Err(_) = tx.send(response).await {
                            tracing::info!("Connection closed");
                        }
                    }
                }
            });

            js.join_next().await;
            js.abort_all();
            if let Err(err) = ctx.router.drop_service(payload.signer).await {
                tracing::warn!("Failed to drop service after ws connection: {err}");
            }
        }
        Err(err) => {
            tracing::warn!("Failed to connect in handle_socket: {err}");
            if let Err(e) = socket
                .send(Message::Text(
                    format!("500: Failed to connect: {err}").into(),
                ))
                .await
            {
                tracing::info!("Connection closed: {e}");
            }
        }
    }
}
