use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents a message frame in transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFrame {
    pub id: String,        // Unique message identifier (UUID v4)
    pub timestamp: u64,    // Unix timestamp in milliseconds
    pub sender_id: String, // Node/client/provider identifier
    pub target_id: Option<String>,
    pub payload: MessagePayload,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Request(Request),
    Response(Response),
    // Heartbeat(HeartbeatPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub service_id: String,
    pub endpoint: Option<String>,
    // pub type_: String, // e.g. "model_completion"
    pub request_type: String,
    pub payload: String,
    pub stream: bool,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub request_id: String,
    pub status_code: u16,
    pub content_type: String,
    pub payload: String,
    pub headers: HashMap<String, String>,
    pub is_stream_chunk: bool,
    pub stream_done: bool,
}
