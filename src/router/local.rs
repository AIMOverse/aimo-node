use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use crate::core::transport::{MessagePayload, Request, Response};
use crate::router::{
    Router,
    interface::{ResponseHandler, make_connection},
};

/// The local transport inplemented with tokio
pub struct LocalRouter {
    client_connections: Arc<Mutex<HashMap<String, mpsc::Sender<Response>>>>,
    service_connections: Arc<Mutex<HashMap<String, mpsc::Sender<Request>>>>,
    message_tx: mpsc::Sender<MessagePayload>,
    message_rx: Arc<Mutex<mpsc::Receiver<MessagePayload>>>,
}

impl LocalRouter {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::channel(128);

        Self {
            client_connections: Arc::new(Mutex::new(HashMap::new())),
            service_connections: Arc::new(Mutex::new(HashMap::new())),
            message_tx,
            message_rx: Arc::new(Mutex::new(message_rx)),
        }
    }

    pub async fn run(&self) {
        let message_rx = self.message_rx.clone();
        let service_connections_ptr = self.service_connections.clone();
        let client_connections_ptr = self.client_connections.clone();

        // Ensure the resource ownership of the runner thread
        let mut message_rx = message_rx.lock().await;

        tracing::info!("Router created and running");
        loop {
            match message_rx.recv().await {
                Some(MessagePayload::Request(request)) => {
                    tracing::debug!("Received message request {:?}", request);
                    let service_id = request.service_id.clone();
                    if let Some(connection) = service_connections_ptr.lock().await.get(&service_id)
                    {
                        let tx = connection.clone();
                        tokio::spawn(async move {
                            if let Err(err) = tx.send(request).await {
                                tracing::warn!("Connection closed: {err}");
                            }
                            tracing::debug!("Forwarded request to service");
                        });
                    } else {
                        tracing::debug!("Service not found");
                    }
                }
                Some(MessagePayload::Response(response)) => {
                    tracing::debug!("Received message response {:?}", response);
                    let request_id = response.request_id.clone();
                    if let Some(connection) = client_connections_ptr.lock().await.get(&request_id) {
                        let tx = connection.clone();
                        tokio::spawn(async move {
                            if let Err(err) = tx.send(response).await {
                                tracing::warn!("Connection closed: {err}");
                            }
                            tracing::debug!("Sent response back to client");
                        });
                    } else {
                        tracing::debug!("Request client {} not found", response.request_id);
                    }
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

#[async_trait]
impl Router for LocalRouter {
    async fn register_service(&self, service_id: String) -> anyhow::Result<ResponseHandler> {
        let (client_handler, service_handler) = make_connection(16, 16);

        let tx = client_handler.tx.clone();
        self.service_connections
            .lock()
            .await
            .insert(service_id.clone(), tx);

        // Send to router's message dispatch system
        let mut rx = client_handler.rx;
        let tx = self.message_tx.clone();
        tokio::spawn(async move {
            while let Some(response) = rx.recv().await {
                if let Err(err) = tx.send(MessagePayload::Response(response)).await {
                    tracing::debug!("Failed to send: {err}");
                    tracing::info!("Service connection lost");
                    break;
                }
            }
            tracing::info!("Service {service_id} disconnected");
        });

        Ok(service_handler)
    }

    async fn route_request(&self, request: Request) -> anyhow::Result<mpsc::Receiver<Response>> {
        let (mut req_handler, res_handler) = make_connection::<Request, _>(1, 1);

        self.client_connections
            .lock()
            .await
            .insert(request.request_id.clone(), res_handler.tx);

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

    async fn drop_service(&self, service_id: String) -> anyhow::Result<()> {
        if self
            .service_connections
            .lock()
            .await
            .remove(&service_id)
            .map(|_| {
                tracing::info!("Service {service_id} dropped");
            })
            .is_none()
        {
            bail!("Failed to drop: Service {service_id} not found");
        }

        Ok(())
    }
}
