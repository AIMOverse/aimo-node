# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AIMO Node is a decentralized AI model routing service that enables direct, real-time, peer-to-peer communication between AI consumers and providers. The system is designed for low-latency, high-throughput streaming of model responses using libp2p transport protocols.

## Development Commands

```bash
# Build the project
cargo build

# Run the project
cargo run

# Build for release
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Apply code formatting
cargo fmt

# Run clippy for linting
cargo clippy
```

## Architecture

This is a Rust project using Cargo as the build system. The current implementation is minimal with:

- **Core Language**: Rust (edition 2024)
- **Transport Layer**: Designed for libp2p (direct streams and gossipsub)
- **Streaming Support**: First-class streaming via HTTP SSE, WebSocket, and libp2p
- **Message Format**: Structured around MessageFrame with support for Request, Response, StreamChunk, Error, and Heartbeat message types

### Key Components (Planned)

Based on the PRD, the system will implement:

1. **Message Frame System**: Core data structures for communication between nodes, clients, and providers
2. **Transport Abstraction**: Supports both libp2p direct streams and gossipsub protocols
3. **Streaming API**: HTTP SSE and WebSocket endpoints for real-time data delivery
4. **Request Routing**: Decentralized routing between AI consumers and providers

### Message Flow

```
[Client] <--HTTP/SSE/WebSocket--> [AIMO Node] <--libp2p stream/pubsub--> [Service Provider]
```

The node acts as a relay and API gateway, with streaming as a first-class feature where responses are delivered to clients as soon as data is available.

## Development Notes

- The project is in early development stage with only a basic main.rs file
- No external dependencies are currently defined in Cargo.toml
- The PRD.md file contains detailed technical specifications for the planned implementation
- The system prioritizes direct libp2p streams for best performance, with pubsub fallback for compatibility