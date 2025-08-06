use std::collections::HashMap;

use anyhow::{Result, anyhow};
use futures_util::{SinkExt, stream::StreamExt};
use reqwest::{Client, Method};
use serde_json;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::router::{ESSENTIAL_RESPONSE_HEADERS, Request, Response, filter_essential_headers};

/// Proxy aimo node requests to standard http endpoints
///
/// 1. Connect to aimo node's websocket endpoint
/// 2. On receiving serialized `Request` messages, spawn a tokio thread to do the following:
///     a. Deserialize the `Request`, extract its payload;
///     b. Forward the request payload to the http endpoint;
///     c. Wait for response.
///     d. If the response is a normal http response, wrap the response in `Response` message,
///         send it back through websocket, and quit the thread.
///     e. If the response is a SSE stream, wrap each chunk inside the `Response` message, and
///         send back through websocket in receiving sequence.
pub async fn serve_websocket(
    node_url: String,
    secret_key: String,
    endpoint_url: String,
    api_key: Option<String>,
) -> anyhow::Result<()> {
    info!("Starting proxy service...");
    info!("Node URL: {}", node_url);
    info!("Endpoint URL: {}", endpoint_url);

    // Parse and build WebSocket URL
    let ws_url = build_websocket_url(&node_url, &secret_key)?;
    info!("Connecting to WebSocket: {}", ws_url);

    // Connect to the node's websocket endpoint
    let url = url::Url::parse(&ws_url)?;

    let mut request = tungstenite::http::Request::builder()
        .method("GET")
        .uri(ws_url.as_str())
        .header("Host", url.host_str().unwrap_or("localhost"))
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header(
            "Sec-WebSocket-Key",
            tungstenite::handshake::client::generate_key(),
        )
        .header("Sec-WebSocket-Version", "13")
        .header("Authorization", format!("Bearer {}", secret_key))
        .body(())?;

    let (ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow!("Failed to connect to WebSocket: {}", e))?;
    info!("WebSocket connection established");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let http_client = Client::new();

    // Create a channel for sending responses back to the websocket
    let (response_tx, mut response_rx) = mpsc::unbounded_channel::<Message>();

    // Spawn task to handle outgoing messages
    let sender_task = tokio::spawn(async move {
        while let Some(message) = response_rx.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Main message loop
    while let Some(message) = ws_receiver.next().await {
        match message {
            Ok(Message::Text(text)) => {
                debug!("Received message: {}", text);

                // Parse the request directly (no MessageFrame wrapper)
                let request: Request = match serde_json::from_str::<Request>(&text) {
                    Ok(req) => {
                        info!(
                            "Parsed request successfully - ID: {}, Method: {}, Type: {}",
                            req.request_id, req.method, req.request_type
                        );
                        req
                    }
                    Err(e) => {
                        warn!("Failed to parse request: {}", e);
                        warn!("Raw message was: {}", text);
                        continue;
                    }
                }; // Clone necessary data for the spawned task
                let endpoint_url = endpoint_url.clone();
                let api_key = api_key.clone();
                let client = http_client.clone();
                let response_sender = response_tx.clone();

                // Spawn a task to handle this request
                tokio::spawn(async move {
                    if let Err(e) =
                        handle_request(client, request, endpoint_url, api_key, response_sender)
                            .await
                    {
                        error!("Error handling request: {}", e);
                    }
                });
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by server");
                break;
            }
            Ok(_) => {
                // Ignore other message types (Binary, Ping, Pong)
                debug!("Received non-text message, ignoring");
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Clean up
    sender_task.abort();
    info!("Proxy service stopped");
    Ok(())
}

/// Build the WebSocket URL with authentication
fn build_websocket_url(node_url: &str, _secret_key: &str) -> Result<String> {
    let mut url = Url::parse(node_url)?;

    // Convert HTTP(S) to WS(S)
    match url.scheme() {
        "http" => url
            .set_scheme("ws")
            .map_err(|_| anyhow!("Invalid scheme"))?,
        "https" => url
            .set_scheme("wss")
            .map_err(|_| anyhow!("Invalid scheme"))?,
        "ws" | "wss" => {} // Already correct
        _ => return Err(anyhow!("Unsupported URL scheme: {}", url.scheme())),
    }

    // Add subscribe endpoint
    url.set_path("/api/v1/providers/subscribe");

    Ok(url.to_string())
}

/// Handle a single request by forwarding it to the HTTP endpoint
async fn handle_request(
    client: Client,
    request: Request,
    endpoint_url: String,
    api_key: Option<String>,
    response_sender: UnboundedSender<Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling request ID: {}", request.request_id);

    // Parse the HTTP method from the new method field
    let method = match request.method.to_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        _ => {
            warn!("Unsupported HTTP method: {}", request.method);
            send_error_response(
                &response_sender,
                &request.request_id,
                400,
                "Unsupported HTTP method",
            )?;
            return Ok(());
        }
    };

    // Build the target URL
    let target_url = if let Some(endpoint) = &request.endpoint {
        format!(
            "{}/{}",
            endpoint_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        )
    } else {
        endpoint_url.clone()
    };

    debug!("Forwarding {} request to: {}", method, target_url);
    debug!("Request headers: {:?}", request.headers);
    debug!("Request payload length: {} bytes", request.payload.len());
    if !request.payload.is_empty() {
        debug!(
            "Request payload preview: {}",
            if request.payload.len() > 200 {
                format!("{}...", &request.payload[..200])
            } else {
                request.payload.clone()
            }
        );
    }

    // Build the HTTP request
    let mut http_request = client.request(method, &target_url);

    // Add headers from the original request
    for (key, value) in &request.headers {
        debug!("Adding header: {}: {}", key, value);
        http_request = http_request.header(key, value);
    }

    // Add API key if provided
    if let Some(api_key) = &api_key {
        debug!("Adding Authorization header with API key");
        http_request = http_request.header("Authorization", format!("Bearer {}", api_key));
    }

    // Add body if present
    if !request.payload.is_empty() {
        // Try to determine content type from headers or default to JSON
        let content_type = request
            .headers
            .get("content-type")
            .or_else(|| request.headers.get("Content-Type"))
            .map(|s| s.as_str())
            .unwrap_or("application/json");

        debug!("Setting content-type to: {}", content_type);
        http_request = http_request
            .header("Content-Type", content_type)
            .body(request.payload);
    }

    debug!("Sending HTTP request...");
    // Send the request
    match http_request.send().await {
        Ok(response) => {
            debug!("Received HTTP response with status: {}", response.status());
            let headers: HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            debug!("Response headers: {:?}", headers);

            // Filter headers to only include essential ones
            let filtered_headers = filter_essential_headers(&headers, ESSENTIAL_RESPONSE_HEADERS);

            let content_type = headers
                .get("content-type")
                .or_else(|| headers.get("Content-Type"))
                .unwrap_or(&"text/plain".to_string())
                .clone();

            // Check if this is a Server-Sent Events stream
            if content_type.contains("text/event-stream") || content_type.contains("text/stream") {
                debug!("Handling SSE stream for request {}", request.request_id);
                handle_sse_stream(
                    response,
                    &response_sender,
                    &request.request_id,
                    &content_type,
                    filtered_headers,
                )
                .await?;
            } else {
                debug!(
                    "Handling regular HTTP response for request {}",
                    request.request_id
                );
                handle_regular_response(
                    response,
                    &response_sender,
                    &request.request_id,
                    &content_type,
                    filtered_headers,
                )
                .await?;
            }
        }
        Err(e) => {
            error!("HTTP request failed: {}", e);
            error!("Error details: {:?}", e);

            // Check if it's a connection error, timeout, etc.
            if e.is_connect() {
                error!("Connection error - unable to connect to {}", target_url);
            } else if e.is_timeout() {
                error!("Request timeout");
            } else if e.is_request() {
                error!("Request construction error");
            } else {
                error!("Other HTTP error type");
            }

            send_error_response(
                &response_sender,
                &request.request_id,
                500,
                &format!("HTTP request failed: {}", e),
            )?;
        }
    }

    Ok(())
}

/// Handle a regular (non-streaming) HTTP response
async fn handle_regular_response(
    response: reqwest::Response,
    response_sender: &UnboundedSender<Message>,
    request_id: &str,
    content_type: &str,
    headers: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let status_code = response.status().as_u16();
    let body = response.text().await.unwrap_or_else(|_| "".to_string());

    let response = Response {
        request_id: request_id.to_string(),
        status_code,
        content_type: content_type.to_string(),
        payload: body,
        headers,
        is_stream_chunk: false,
        stream_done: true,
    };

    let message = Message::text(serde_json::to_string(&response)?);
    response_sender.send(message)?;

    debug!("Sent regular response for request {}", request_id);
    Ok(())
}

/// Handle a Server-Sent Events (SSE) stream response
async fn handle_sse_stream(
    response: reqwest::Response,
    response_sender: &UnboundedSender<Message>,
    request_id: &str,
    content_type: &str,
    headers: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let status_code = response.status().as_u16();
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                let chunk_data = String::from_utf8_lossy(&chunk).to_string();

                // Send this chunk as a streaming response
                let response = Response {
                    request_id: request_id.to_string(),
                    status_code,
                    content_type: content_type.to_string(),
                    payload: chunk_data,
                    headers: headers.clone(),
                    is_stream_chunk: true,
                    stream_done: false,
                };

                let message = Message::text(serde_json::to_string(&response)?);
                if response_sender.send(message).is_err() {
                    error!("Failed to send stream chunk, connection closed");
                    break;
                }

                debug!("Sent stream chunk for request {}", request_id);
            }
            Err(e) => {
                error!("Error reading stream chunk: {}", e);
                break;
            }
        }
    }

    // Send final "stream done" message
    let final_response = Response {
        request_id: request_id.to_string(),
        status_code,
        content_type: content_type.to_string(),
        payload: "".to_string(),
        headers,
        is_stream_chunk: true,
        stream_done: true,
    };

    let message = Message::text(serde_json::to_string(&final_response)?);
    response_sender.send(message)?;

    debug!("Stream completed for request {}", request_id);
    Ok(())
}

/// Send an error response back through the WebSocket
fn send_error_response(
    response_sender: &UnboundedSender<Message>,
    request_id: &str,
    status_code: u16,
    error_message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = Response {
        request_id: request_id.to_string(),
        status_code,
        content_type: "text/plain".to_string(),
        payload: error_message.to_string(),
        headers: HashMap::new(),
        is_stream_chunk: false,
        stream_done: true,
    };

    let message = Message::text(serde_json::to_string(&response)?);
    response_sender.send(message)?;

    Ok(())
}
