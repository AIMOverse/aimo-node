# AIMO Node: Decentralized OpenRouter - Product Requirements Document

## Executive Summary

AIMO Node is a decentralized AI model routing service that enables direct peer-to-peer communication between AI service consumers and providers. Unlike centralized platforms, AIMO creates a distributed network where nodes act as message relays using libp2p gossipsub protocol, facilitating real-time chat completion requests and responses with streaming support.

**Key Value Proposition**: Eliminate intermediary fees, reduce latency, and provide censorship-resistant AI model access through decentralized infrastructure.

## Product Vision

Create a robust, scalable node service that serves as the foundation for a decentralized AI model marketplace, where service providers can offer their models directly to consumers through a trustless, peer-to-peer network.

## Core Requirements

### 1. System Architecture Overview

```
[Client] <---> [AIMO Node] <---> [libp2p gossipsub network] <---> [Service Provider]
```

**CRITICAL IMPLEMENTATION NOTE FOR AI AGENTS**: This is a single-node implementation focusing on message routing. The node acts as both a relay server and API gateway. Peer discovery is NOT required in this version.

### 2. Message Frame Data Structure

#### 2.1 Base Message Frame
```rust
// IMPLEMENTATION REQUIREMENT: Use this exact structure
pub struct MessageFrame {
    pub id: String,              // Unique message identifier (UUID v4)
    pub timestamp: u64,          // Unix timestamp in milliseconds
    pub message_type: MessageType,
    pub sender_id: String,       // Node/client/provider identifier
    pub target_id: Option<String>, // Optional target for direct routing
    pub payload: MessagePayload,
    pub signature: Option<String>, // For future authentication
}

pub enum MessageType {
    Request,
    Response,
    Stream,
    Error,
    Heartbeat,
}
```

#### 2.2 Generic Request/Response Payload
```rust
// IMPLEMENTATION REQUIREMENT: Generic payload for flexible routing
pub enum MessagePayload {
    Request(Request),
    Response(Response),
    StreamChunk(StreamChunk),
    Error(ErrorPayload),
    Heartbeat(HeartbeatPayload),
}


pub struct Request {
    pub provider_id: String,     // Target service provider identifier
    pub endpoint: Option<String>, // Optional specific endpoint
    pub type: String,           // Request type (e.g., "model_completion", "health_check", etc.)
    pub content_type: String,    // MIME type of the payload (e.g., "application/json")
    pub payload: String,         // Raw request data as string
    pub stream: bool,            // Enable streaming responses
    pub headers: HashMap<String, String>, // Optional headers for the request
}

pub struct Response {
    pub request_id: String,      // Original request identifier
    pub status_code: u16,        // HTTP-like status code
    pub content_type: String,    // MIME type of the response
    pub payload: String,         // Raw response data as string
    pub headers: HashMap<String, String>, // Response headers
    pub is_stream_chunk: bool,   // Indicates if this is part of a stream
    pub stream_done: bool,       // Indicates stream completion
}

pub struct StreamChunk {
    pub request_id: String,      // Original request identifier
    pub chunk_index: u32,        // Sequential chunk number
    pub content_type: String,    // MIME type of the chunk
    pub payload: String,         // Raw chunk data as string
    pub is_final: bool,          // Indicates if this is the last chunk
}
```

#### 2.3 Error and Heartbeat Payloads
```rust
// IMPLEMENTATION REQUIREMENT: Support error reporting and health monitoring
pub struct ErrorPayload {
    pub error_code: String,      // Error identifier
    pub message: String,         // Human-readable error message
    pub details: Option<String>, // Additional error context
    pub request_id: Option<String>, // Associated request if applicable
}

pub struct HeartbeatPayload {
    pub node_id: String,         // Node identifier
    pub status: String,          // "healthy", "degraded", "offline"
    pub load: Option<f32>,       // Current load percentage (0.0-1.0)
    pub capabilities: Vec<String>, // Supported features
}
```

### 3. Request Routing Workflow

#### 3.1 Client Request Flow
```
IMPLEMENTATION SEQUENCE (CRITICAL FOR AI AGENTS):

1. Client sends generic request to AIMO Node via REST API (any format/content-type)
2. Node validates basic request structure and extracts provider_id
3. Node wraps request in MessageFrame with type=Request and GenericRequest payload
4. Node publishes message to gossipsub topic: "aimo/requests/{provider_id}/{type}"
5. Node subscribes to response topic: "aimo/responses/{request_id}"
6. Node waits for response or timeout (30 seconds default)
7. Node forwards raw response back to client via HTTP with original content-type
```

#### 3.2 Service Provider Flow
```
IMPLEMENTATION SEQUENCE (CRITICAL FOR AI AGENTS):

1. Service provider connects directly to the relay (AIMO Node) via gossipsub
2. Service provider subscribes to its own request topic: "aimo/requests/{provider_id}/*" or "aimo/requests/{provider_id}/{type}"
3. Service provider receives MessageFrame with Request payload
4. Service provider extracts payload and processes using its own logic/format
5. Service provider publishes Response to topic: "aimo/responses/{request_id}"
6. For streaming: Service provider sends multiple StreamChunk frames to "aimo/responses/{request_id}"
7. Service provider sends final chunk with is_final=true
```

#### 3.3 Streaming Response Handling
```rust
// IMPLEMENTATION REQUIREMENT: Support real-time streaming
pub struct StreamManager {
    active_streams: HashMap<String, StreamContext>,
    chunk_buffer: HashMap<String, Vec<StreamChunk>>,
}

pub struct StreamContext {
    request_id: String,
    client_connection: ConnectionId,
    start_time: u64,
    chunk_count: u32,
    is_complete: bool,
    content_type: String,        // Track content type for proper client response
}
```

### 4. Technical Implementation Requirements

#### 4.1 libp2p gossipsub Configuration
```rust
// MANDATORY CONFIGURATION FOR AI AGENTS
let gossipsub_config = GossipsubConfigBuilder::default()
    .max_transmit_size(1024 * 1024) // 1MB max message size
    .heartbeat_interval(Duration::from_secs(1))
    .validation_mode(ValidationMode::Strict)
    .message_id_fn(|message| {
        // Use message frame ID for deduplication
        MessageId::from(message.data.clone())
    })
    .build()
    .expect("Valid config");
```

#### 4.2 Topic Naming Convention
```
CRITICAL NAMING PATTERN FOR AI AGENTS:
- Request topics: "aimo/requests/{provider_id}/{type}" (service providers subscribe to their own provider_id and/or type)
- Response topics: "aimo/responses/{request_id}"
- Heartbeat topics: "aimo/heartbeat"
- Error topics: "aimo/errors"
```

#### 4.3 REST API Endpoints
```rust
// MANDATORY API ENDPOINTS FOR AI AGENTS TO IMPLEMENT
POST /v1/request/{provider_id}   // Generic request routing to provider
GET  /v1/providers              // List available providers
POST /v1/providers/register     // Provider registration
GET  /v1/health                 // Node health check
WS   /v1/stream                 // WebSocket for real-time updates

// LEGACY COMPATIBILITY (OPTIONAL)
POST /v1/chat/completions       // OpenAI-compatible passthrough
```

#### 4.4 Error Handling Requirements
```rust
// IMPLEMENTATION REQUIREMENT: Comprehensive error handling
pub enum NodeError {
    RequestTimeout,
    ProviderNotFound,
    InvalidMessageFormat,
    NetworkError(String),
    StreamInterrupted,
    RateLimitExceeded,
    PayloadTooLarge,
    UnsupportedContentType,
}
```

### 5. Performance and Reliability Requirements

#### 5.1 Latency Targets
- **Request routing**: < 10ms
- **End-to-end response**: < 2 seconds (non-streaming)
- **Stream chunk delivery**: < 100ms

#### 5.2 Throughput Requirements
- **Concurrent requests**: 1,000+ per node
- **Message throughput**: 10,000 messages/second
- **Stream concurrency**: 100+ simultaneous streams

#### 5.3 Reliability Targets
- **Uptime**: 99.9%
- **Message delivery**: 99.99% success rate
- **Request timeout**: 30 seconds maximum

### 6. Security Considerations

#### 6.1 Message Validation
```rust
// SECURITY REQUIREMENT FOR AI AGENTS
pub fn validate_message_frame(frame: &MessageFrame) -> Result<(), ValidationError> {
    // Validate message size limits (max 1MB payload)
    // Check timestamp freshness (within 60 seconds)
    // Verify sender_id format
    // Validate payload structure (basic envelope validation only)
    // Content validation is delegated to service providers
}
```

#### 6.2 Rate Limiting
- **Per client**: 100 requests/minute
- **Per provider**: 1,000 requests/minute
- **Global node**: 10,000 requests/minute

### 7. Development Phases

#### Phase 1: Core Message Routing (Current)
- [ ] Implement MessageFrame and generic payload structures
- [ ] Set up libp2p gossipsub network
- [ ] Create REST API layer for generic request routing
- [ ] Implement basic request/response routing with raw payload passthrough
- [ ] Add streaming support for chunked responses

#### Phase 2: Future Enhancements
- Decentralized identity authentication
- Statistics monitoring and analytics
- On-chain payment processing (Solana)
- Multi-node peer discovery
- Advanced load balancing

### 8. Testing Strategy

#### 8.1 Unit Tests
- Message serialization/deserialization (generic payloads)
- Request routing logic (provider_id extraction)
- Stream management (chunk ordering and completion)
- Error handling scenarios (malformed payloads, timeouts)

#### 8.2 Integration Tests
- End-to-end request flow (various content types)
- Multiple provider scenarios (different payload formats)
- Stream interruption handling (partial chunk delivery)
- Network partition recovery (message retry logic)

#### 8.3 Performance Tests
- Load testing with 1,000+ concurrent requests
- Stream performance under load
- Memory usage optimization
- Network bandwidth efficiency

### 9. Implementation Notes for AI Agents

**CRITICAL ATTENTION POINTS**:

1. **Use Rust's tokio async runtime** for all networking operations
2. **Implement proper connection pooling** for libp2p peers
3. **Use structured logging** with tracing crate for debugging
4. **Implement graceful shutdown** handling for all services
5. **Use configuration management** with environment variables
6. **Implement comprehensive metrics collection** using prometheus
7. **Follow Rust naming conventions** and use `cargo fmt` for formatting
8. **Add proper documentation** with `cargo doc` comments
9. **Implement proper error propagation** using `Result<T, E>` types
10. **Use dependency injection** patterns for testability

### 10. Success Metrics

#### 10.1 Technical Metrics
- Message delivery success rate > 99.99%
- Average response latency < 500ms
- Zero message corruption or loss
- Stream completion rate > 99%

#### 10.2 Business Metrics
- Number of active providers
- Request volume growth
- Client retention rate
- Network effect expansion

---

**IMPORTANT FOR AI IMPLEMENTATION**: This PRD serves as the single source of truth for the AIMO Node implementation. All code must strictly adhere to the data structures, workflows, and requirements specified above. Any deviations must be justified and documented.