# Envoy Body-to-Header Dynamic Module

A proof-of-concept Envoy dynamic module written in Rust that demonstrates logging at each stage of HTTP request processing.

## Overview

This module implements a simple passthrough filter that logs detailed information about:
- Request headers
- Request body 
- Request trailers
- Response headers
- Response body
- Response trailers

All logs are prefixed with `[BODY_TO_HEADER]` and include a request ID for tracing.

## Structure

```
├── src/
│   ├── lib.rs                     # Main module entry point
│   └── logging_passthrough.rs     # Logging passthrough filter implementation
├── .cargo/
│   └── config.toml               # macOS compatibility configuration
├── Cargo.toml                    # Rust project configuration
├── Dockerfile                    # Multi-arch Docker build for Envoy + module
├── docker-compose.yml            # Docker Compose setup (Envoy + httpbin)
├── envoy.yaml                    # Envoy configuration for testing
└── README.md                     # This file
```

## Building

### Build and Run with Docker Compose

```bash
# Build and start both Envoy and httpbin
docker-compose up --build

# Run in detached mode
docker-compose up --build -d

# View logs
docker-compose logs -f envoy
```

### Build Docker Image Only

```bash
docker buildx build . -t envoy-body-to-header:latest
```

For multi-arch builds:
```bash
docker buildx build . -t envoy-body-to-header:latest --platform linux/amd64,linux/arm64
```

### Build Rust Module Locally

```bash
cargo build
```

For release build:
```bash
cargo build --release
```

## Testing

### Start the Services

```bash
docker-compose up --build
```

### Test the Module

Send test requests to see the logging in action:

```bash
# Simple GET request
curl http://localhost:8080/get

# POST request with body
curl -X POST http://localhost:8080/post \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello from the body!"}'

# Request with custom headers
curl http://localhost:8080/headers \
  -H "X-Custom-Header: test-value" \
  -H "X-Another-Header: another-value"
```

### View Logs

The module logs will appear in the Docker container output with the `[BODY_TO_HEADER]` prefix.

```bash
# View all logs
docker-compose logs

# Follow Envoy logs specifically
docker-compose logs -f envoy

# Follow httpbin logs
docker-compose logs -f httpbin
```

### Stop Services

```bash
# Stop and remove containers
docker-compose down

# Stop, remove containers, and remove images
docker-compose down --rmi all
```

### Envoy Admin Interface

Access the Envoy admin interface at http://localhost:9901 for additional debugging and metrics.

## Development

### Run Tests

```bash
cargo test
```

### Format Code

```bash
cargo fmt
```

### Lint Code

```bash
cargo clippy -- -D warnings
```

## Configuration

The module is configured in `envoy.yaml` as a dynamic module filter:

```yaml
- name: dynamic_modules/body_to_header
  typed_config:
    "@type": type.googleapis.com/envoy.extensions.filters.http.dynamic_modules.v3.DynamicModuleFilter
    dynamic_module_config:
      name: body_to_header_module
    filter_name: logging_passthrough
    filter_config:
      "@type": "type.googleapis.com/google.protobuf.StringValue"
      value: |
        {
          "debug": true
        }
```

### Available Services

- **Envoy Proxy**: http://localhost:8080 (proxies to httpbin)
- **Envoy Admin**: http://localhost:9901 (admin interface)
- **httpbin Direct**: http://localhost:8000 (for comparison)

> **Note**: Using `mccutchen/go-httpbin` for better cross-platform compatibility (especially Apple Silicon Macs).

## Next Steps

This is a proof-of-concept. Future iterations might include:
- Extracting data from request bodies
- Adding extracted data as response headers
- Configuration options for what to extract
- Performance optimizations 