# AiMo Node Proxy Service

The proxy service allows you to connect your HTTP endpoints directly to the AiMo Network without implementing the full AiMo Network protocol.

## How it works

1. The proxy connects to an AiMo Network node via WebSocket
2. It receives `Request` messages from the node
3. Forwards these requests to your HTTP endpoint
4. Sends back responses through the WebSocket connection
5. Supports both regular HTTP responses and Server-Sent Events (SSE) streaming

## Usage

```bash
aimo proxy \
  --node-url "http://localhost:8000" \
  --secret-key "aimo-sk-dev-xxxx" \
  --endpoint-url "http://localhost:3000" \
  --api-key "your-api-key-if-needed"
```

### Parameters

- `--node-url`: URL of the AiMo Network node to connect to
- `--secret-key`: Your AiMo Network secret key (generate with `aimo keygen`)
- `--endpoint-url`: Your HTTP service endpoint URL
- `--api-key`: Optional API key for your service endpoint

## Features

### HTTP Method Support

- GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS

### Header Forwarding

- All headers from the original request are forwarded to your endpoint
- Optional API key is added as `Authorization: Bearer <key>` header

### Content Type Detection

- Automatically detects content type from headers
- Defaults to `application/json` if not specified

### Streaming Support

- Automatically detects Server-Sent Events (`text/event-stream`)
- Streams responses chunk by chunk back to the client
- Proper stream completion signaling

### Error Handling

- HTTP request failures are properly propagated
- Unsupported HTTP methods return 400 errors
- Connection errors are logged and handled gracefully

## Example

1. Start your HTTP service on `http://localhost:3000`
2. Generate a secret key: `aimo keygen --tag dev`
3. Start the proxy:

   ```bash
   aimo proxy \
     --node-url "http://localhost:8000" \
     --secret-key "aimo-sk-dev-xxxxx" \
     --endpoint-url "http://localhost:3000"
   ```

4. Your service is now accessible through the AiMo Network!

## Troubleshooting

- Ensure your AiMo node is running and accessible
- Verify your secret key is valid
- Check that your endpoint URL is reachable
- Monitor logs for connection and forwarding errors
