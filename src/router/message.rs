use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents a message frame in transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFrame {
    pub id: String,     // Unique message identifier (UUID v4)
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub target_id: Option<String>,
    pub payload: MessagePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Request(Request),
    Response(Response),
    // Heartbeat(HeartbeatPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub sender_id: String, // client id / public key
    pub request_id: String,
    pub service_id: String,
    pub endpoint: Option<String>,
    pub request_type: String,
    pub payload: String,
    pub headers: HashMap<String, String>,
    pub payload_encrypted: bool,
    pub signature: Option<String>,
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
