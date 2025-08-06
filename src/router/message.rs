use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents a message frame in transport (for internal use)
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
    pub request_type: String, // Resource type: "completion_model", "embedding_model", etc.
    pub method: String,       // HTTP method: "GET", "POST", "PUT", "DELETE", etc.
    pub payload: String,
    pub headers: HashMap<String, String>, // Only essential headers
    pub payload_encrypted: bool,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub request_id: String,
    pub status_code: u16,
    pub content_type: String,
    pub payload: String,
    pub headers: HashMap<String, String>, // Only essential headers
    pub is_stream_chunk: bool,
    pub stream_done: bool,
}

/// Headers that should be included in requests to reduce message size
pub const ESSENTIAL_REQUEST_HEADERS: &[&str] = &[
    "content-type",
    "content-length",
    "authorization",
    "accept",
    "accept-encoding",
    "user-agent",
    "x-forwarded-for",
    "x-real-ip",
];

/// Headers that should be included in responses to reduce message size
pub const ESSENTIAL_RESPONSE_HEADERS: &[&str] = &[
    "content-type",
    "content-length",
    "content-encoding",
    "cache-control",
    "expires",
    "last-modified",
    "etag",
    "access-control-allow-origin",
    "access-control-allow-methods",
    "access-control-allow-headers",
];

/// Filter headers to only include essential ones
pub fn filter_essential_headers(
    headers: &HashMap<String, String>,
    essential: &[&str],
) -> HashMap<String, String> {
    headers
        .iter()
        .filter(|(key, _)| {
            let key_lower = key.to_lowercase();
            essential
                .iter()
                .any(|&essential_key| key_lower == essential_key)
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}
