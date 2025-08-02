# AIMO Node: Decentralized OpenRouter – Product Requirements Document (Streaming-Optimized)

## Executive Summary

AIMO Node is a decentralized AI model routing service enabling direct, real-time, peer-to-peer communication between AI consumers and providers. The system is designed for low-latency, high-throughput streaming of model responses, supporting both chunked and continuous data delivery. The transport layer is modular: gossipsub, direct libp2p streams, or other protocols may be used as appropriate.

## Product Vision

Build a robust, extensible node service that forms the backbone of a decentralized AI model marketplace, with first-class support for streaming, flexible transport, and future extensibility (auth, payments, analytics).

## Core Requirements

### 1. System Architecture Overview

```
[Client] <--HTTP/SSE/WebSocket--> [AIMO Node] <--libp2p stream/pubsub/other--> [Service Provider]
```

- The node acts as a relay and API gateway.
- Streaming is a first-class feature: responses are delivered to clients as soon as data is available.

### 2. Message Frame Data Structure

#### 2.1 Base Message Frame

```rust
pub struct MessageFrame {
    pub id: String,              // Unique message identifier (UUID v4)
    pub timestamp: u64,          // Unix timestamp in milliseconds
    pub message_type: MessageType,
    pub sender_id: String,       // Node/client/provider identifier
    pub target_id: Option<String>,
    pub payload: MessagePayload,
    pub signature: Option<String>,
}

pub enum MessageType {
    Request,
    Response,
    StreamChunk,
    Error,
    Heartbeat,
}
```

#### 2.2 Request/Response/Stream Payloads

```rust
pub enum MessagePayload {
    Request(Request),
    Response(Response),
    StreamChunk(StreamChunk),
    Error(ErrorPayload),
    Heartbeat(HeartbeatPayload),
}

pub struct Request {
    pub provider_id: String,
    pub endpoint: Option<String>,
    pub type_: String,           // e.g. "model_completion"
    pub content_type: String,
    pub payload: String,
    pub stream: bool,
    pub headers: HashMap<String, String>,
}

pub struct Response {
    pub request_id: String,
    pub status_code: u16,
    pub content_type: String,
    pub payload: String,
    pub headers: HashMap<String, String>,
    pub is_stream_chunk: bool,
    pub stream_done: bool,
}

pub struct StreamChunk {
    pub request_id: String,
    pub chunk_index: u32,
    pub content_type: String,
    pub payload: String,
    pub is_final: bool,
}
```

### 3. Streaming Workflow

#### 3.1 Client Request Flow

1. Client sends a request (REST, WebSocket, or SSE) to the node, indicating if streaming is desired.
2. Node validates and wraps the request in a MessageFrame.
3. Node establishes a stream to the provider (using libp2p direct stream, pubsub, or other).
4. As the provider sends chunks, the node forwards each chunk to the client immediately (SSE or WebSocket).
5. Node closes the stream when the provider signals completion.

#### 3.2 Provider Flow

1. Provider connects to the node using a streaming-capable protocol (libp2p direct stream preferred for low-latency).
2. Provider subscribes to its own request topics or opens a direct stream.
3. Provider receives requests, processes them, and streams responses chunk-by-chunk.
4. Provider signals stream completion with a final chunk.

#### 3.3 Topic/Stream Design

- If using pubsub: topics are `"aimo/requests/{provider_id}/{type}"` and `"aimo/responses/{request_id}"`.
- If using direct streams: node opens a stream per request, and provider streams data back on the same connection.
- The node should support both, with a preference for direct streams for high-throughput/low-latency use cases.

### 4. Technical Implementation Requirements

- **Transport Abstraction**: Implement a transport layer that can use gossipsub, direct libp2p streams, or other protocols.
- **Streaming API**: Expose HTTP SSE and WebSocket endpoints for clients to receive streamed responses.
- **Backpressure Handling**: Implement flow control to avoid overwhelming clients or providers.
- **Chunk Ordering & Reliability**: Ensure chunks are delivered in order and handle retransmission or error signaling if needed.
- **Max Chunk Size**: Enforce a reasonable max chunk size (e.g., 64KB) to fit within pubsub or stream message limits.

### 5. Security & Extensibility

- **Authentication**: Pluggable, to be added in future versions.
- **Rate Limiting**: Per-client and per-provider.
- **Metrics**: Track stream latency, chunk delivery, and errors.

### 6. Testing

- Simulate high-concurrency streaming scenarios.
- Test both pubsub and direct stream transports.
- Validate chunk ordering, loss, and recovery.

---

**Summary:**  
This design makes streaming a first-class feature, allows for both pubsub and direct stream transports, and ensures that clients receive streamed responses in real time (e.g., via SSE). The node should default to direct libp2p streams for best performance, but fall back to pubsub if needed for compatibility or broadcast scenarios.
