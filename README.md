# Envoy Body-Based Routing Dynamic Module

Routes HTTP requests to different upstream clusters based on request body content.

Analyzes JSON request bodies and sets routing headers. Default route is `echo1`.

## Quick Start

```bash
# Build and start
docker-compose up --build

# Test routing to echo2
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"method": "echo2", "data": "test"}'

# Test default routing to echo1
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"method": "anything", "data": "test"}'
```

## Services

- **Envoy Proxy**: `http://localhost:8080` 
- **Echo Server 1**: `http://localhost:8001` (default route)
- **Echo Server 2**: `http://localhost:8002` (when method contains "echo2")

## Build

```bash
cargo build
cargo test
```

See `envoy.yaml` for configuration details. 