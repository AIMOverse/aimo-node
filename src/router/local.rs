use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, mpsc};

use crate::router::{
    MessagePayload, Request, Response, Router,
    interface::{RequestHandler, ResponseHandler, make_connection},
};

/// The local transport inplemented with tokio
pub struct LocalRouter {
    client_connection_handlers: Arc<Mutex<HashMap<String, ResponseHandler>>>,
    service_connection_handlers: Arc<Mutex<HashMap<String, RequestHandler>>>,
    message_tx: mpsc::Sender<MessagePayload>,
    message_rx: Arc<Mutex<mpsc::Receiver<MessagePayload>>>,
}

impl LocalRouter {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::channel(128);

        Self {
            client_connection_handlers: Arc::new(Mutex::new(HashMap::new())),
            service_connection_handlers: Arc::new(Mutex::new(HashMap::new())),
            message_tx,
            message_rx: Arc::new(Mutex::new(message_rx)),
        }
    }

    pub async fn run(&self) {
        let message_rx = self.message_rx.clone();
        let service_connections_ptr = self.service_connection_handlers.clone();
        let client_connections_ptr = self.client_connection_handlers.clone();

        // Ensure the resource ownership of the runner thread
        let mut message_rx = message_rx.lock().await;

        tracing::info!("Router created and running");
        loop {
            match message_rx.recv().await {
                Some(MessagePayload::Request(request)) => {
                    let service_id = request.service_id.clone();
                    service_connections_ptr
                        .lock()
                        .await
                        .get(&service_id)
                        .map(|conn| {
                            let tx = conn.tx.clone();
                            tokio::spawn(async move {
                                if let Err(err) = tx.send(request).await {
                                    tracing::warn!("Connection closed: {err}");
                                }
                            });
                        });
                }
                Some(MessagePayload::Response(response)) => {
                    let request_id = response.request_id.clone();
                    client_connections_ptr
                        .lock()
                        .await
                        .get(&request_id)
                        .map(|conn| {
                            let tx = conn.tx.clone();
                            tokio::spawn(async move {
                                if let Err(err) = tx.send(response).await {
                                    tracing::warn!("Connection closed: {err}");
                                }
                            })
                        });
                }
                None => {
                    tracing::error!(
                        "Failed to receive from incoming_rx: channel closed unexpectedly"
                    );
                    break;
                }
            }
        }

        tracing::error!("Router incoming request listener closed unexpectedly");
    }
}

impl LocalRouter {}

impl Router for LocalRouter {
    async fn register_service(&self, service_id: String) -> anyhow::Result<ResponseHandler> {
        let (client_handler, service_handler) = make_connection(16, 16);

        self.service_connection_handlers
            .lock()
            .await
            .insert(service_id, client_handler);

        Ok(service_handler)
    }

    async fn route_request(
        &self,
        request_id: String,
        request: Request,
    ) -> anyhow::Result<mpsc::Receiver<Response>> {
        let (mut req_handler, res_handler) = make_connection(1, 1);

        self.client_connection_handlers
            .lock()
            .await
            .insert(request_id, res_handler);

        // req_handler.send(request).await?;
        // For now, use the public incoming channel directly for sending requests.
        self.message_tx
            .send(MessagePayload::Request(request))
            .await?;

        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            while let Ok(resp) = req_handler.recv().await {
                if let Err(err) = tx.send(resp).await {
                    tracing::debug!("Failed to send response back to client: {err}");
                    tracing::info!("Client connection closed");
                    break;
                }
            }
        });

        Ok(rx)
    }
}
