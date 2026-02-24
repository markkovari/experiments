# HTTP KV Counter - wasmCloud Component

A wasmCloud component written in Rust that provides an HTTP API for managing counters stored in a NATS KV store with a 3-second TTL.

## Features

- **HTTP API** for counter management
  - `GET /` - Returns all counters as JSON array
  - `GET /:name` - Returns a specific counter value
  - `POST /:name` - Increments counter (creates with value 1 if not exists)
- **NATS KV Store** with automatic 3-second TTL expiration
- **Atomic operations** for thread-safe counter increments
- **Comprehensive testing** (unit, integration, and e2e tests)
- **Docker-based development** - no local NATS/wash installation required!

## 🐳 Quick Start with Docker (Recommended)

**Prerequisites:** Only Docker and Docker Compose required!

```bash
# 1. Start all services (NATS + wasmCloud)
make docker-up

# 2. Build and deploy the application
make docker-deploy

# 3. Test the API
curl -X POST http://localhost:8080/mycounter
# {"name":"mycounter","value":1}

curl http://localhost:8080/mycounter
# {"name":"mycounter","value":1}

# 4. Run tests
make docker-test

# 5. Stop everything
make docker-down
```

### Or use the all-in-one command:

```bash
make docker-run    # Starts everything and deploys the app
```

### Available Docker Commands

```bash
make docker-up           # Start NATS + wasmCloud
make docker-down         # Stop all services
make docker-build        # Build the component
make docker-deploy       # Deploy to wasmCloud
make docker-run          # Start and deploy everything
make docker-test         # Run all tests
make docker-test-e2e     # Run e2e tests only
make docker-logs         # Show all logs
make docker-clean        # Clean everything
```

## Prerequisites (for local development)

**Note:** If using Docker (recommended), you only need Docker and Docker Compose. Skip this section!

For local development without Docker:

1. **Rust toolchain** (1.75+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add wasm32-wasip2
   ```

2. **wasmCloud Shell (wash)**
   ```bash
   cargo install wash-cli
   ```

3. **NATS Server** with JetStream
   ```bash
   # macOS
   brew install nats-server

   # Linux
   curl -L https://github.com/nats-io/nats-server/releases/download/v2.10.7/nats-server-v2.10.7-linux-amd64.tar.gz | tar xz
   sudo mv nats-server-v2.10.7-linux-amd64/nats-server /usr/local/bin/
   ```

## Building (Local Development)

**With Docker:**
```bash
make docker-build
```

**Without Docker:**
```bash
wash build
# The compiled component will be in: build/http_kv_counter_s.wasm
```

## Running (Local Development)

**Note:** For Docker-based workflow, see "Quick Start with Docker" above.

### 1. Start NATS Server with JetStream

```bash
nats-server -js
```

### 2. Start wasmCloud Host

```bash
wash up
```

### 3. Deploy the Application

```bash
wash app deploy wadm.yaml
```

The HTTP server will be available at `http://localhost:8080`

## Testing

### With Docker (Recommended)

```bash
# Run all tests (unit + integration + e2e)
make docker-test-all

# Or step by step
make docker-up          # Start services
make docker-deploy      # Deploy app
make docker-test-e2e    # Run e2e tests
make docker-down        # Clean up
```

### Without Docker (Local Development)

#### Unit Tests

Tests for core logic (path parsing, serialization, etc.):

```bash
cargo test --lib
```

#### Integration Tests

Tests for business logic without full wasmCloud runtime:

```bash
cargo test --test integration_test
```

#### End-to-End Tests

Tests with actual wasmCloud runtime and NATS server (requires running services):

```bash
# Start NATS and wasmCloud first (see Running section)
cargo test --test e2e_test -- --ignored
```

## API Examples

### Create/Increment a Counter

```bash
curl -X POST http://localhost:8080/mycounter
# Response: {"name":"mycounter","value":1}

curl -X POST http://localhost:8080/mycounter
# Response: {"name":"mycounter","value":2}
```

### Get a Specific Counter

```bash
curl http://localhost:8080/mycounter
# Response: {"name":"mycounter","value":2}
```

### Get All Counters

```bash
curl http://localhost:8080/
# Response: [{"name":"mycounter","value":2},{"name":"another","value":5}]
```

### TTL Behavior

After 3 seconds of inactivity, counters are automatically deleted:

```bash
curl -X POST http://localhost:8080/temp
# Response: {"name":"temp","value":1}

sleep 4

curl http://localhost:8080/temp
# Response: {"error":"Counter not found"}
```

## Project Structure

```
.
├── Cargo.toml              # Rust dependencies
├── wasmcloud.toml          # wasmCloud component config
├── wadm.yaml               # Deployment manifest
├── wit/                    # WebAssembly Interface definitions
│   ├── world.wit           # Component interface definition
│   └── deps.toml           # WIT dependencies
├── src/
│   ├── lib.rs              # Main component implementation
│   └── bindings.rs         # Generated WIT bindings
└── tests/
    ├── integration_test.rs # Integration tests
    └── e2e_test.rs         # End-to-end tests
```

## Architecture

### Component Architecture
```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ HTTP
       ▼
┌─────────────────────────┐
│  HTTP Server Provider   │
└──────┬──────────────────┘
       │ wasi:http
       ▼
┌─────────────────────────┐
│  Counter Component      │
│  (This component)       │
└──────┬──────────────────┘
       │ wasi:keyvalue
       ▼
┌─────────────────────────┐
│  NATS KV Provider       │
└──────┬──────────────────┘
       │ NATS Protocol
       ▼
┌─────────────────────────┐
│  NATS Server            │
│  (with JetStream KV)    │
└─────────────────────────┘
```

### Docker Architecture
```
┌──────────────────────────────────────────┐
│           Docker Compose Network         │
│                                          │
│  ┌────────────┐    ┌─────────────────┐  │
│  │    NATS    │◄───┤  wasmCloud Host │  │
│  │ JetStream  │    │   + Component   │  │
│  └────────────┘    └────────┬────────┘  │
│                              │           │
│                              │ HTTP      │
│                    ┌─────────▼────────┐  │
│                    │   Port 8080      │  │
│                    └──────────────────┘  │
└──────────────────────────────────────────┘
                     │
                     ▼
              ┌─────────────┐
              │   Client    │
              └─────────────┘
```

## Configuration

### TTL Settings

The 3-second TTL is configured in `wadm.yaml`:

```yaml
source_config:
  - name: counters
    properties:
      bucket: counters
      ttl: 3
      max_age: 3s
```

## Troubleshooting

### Docker Issues

**Services not starting:**
```bash
# Check service status
docker compose ps

# View logs
make docker-logs

# Restart everything
make docker-restart
```

**Port already in use (8080, 4222, etc.):**
```bash
# Find and kill the process using the port
lsof -ti:8080 | xargs kill -9

# Or use different ports by editing docker-compose.yml
```

**Application not responding:**
```bash
# Check wasmCloud logs
make docker-logs-wasmcloud

# Check NATS logs
make docker-logs-nats

# Verify services are healthy
docker compose ps
```

### Local Development Issues

**"Failed to open bucket":**

Ensure NATS server is running with JetStream enabled:
```bash
nats-server -js
```

**Build Errors:**

Ensure you have the wasm32-wasip2 target installed:
```bash
rustup target add wasm32-wasip2
```

**E2E Tests Failing:**

1. Verify wasmCloud host is running: `wash get hosts`
2. Verify application is deployed: `wash app list`
3. Check logs: `wash app logs http-kv-counter`

## Development

### Docker-Based Development (Recommended)

```bash
# Start services
make docker-up

# Build and deploy
make docker-deploy

# Watch logs
make docker-logs

# Make changes to code, then rebuild
make docker-build
make docker-deploy

# Clean up
make docker-down
```

### Local Development with Hot Reload

Use `wash dev` for automatic rebuilding and redeployment:

```bash
wash dev
```

### Cleaning Up

**With Docker:**
```bash
make docker-clean  # Removes containers, volumes, and build artifacts
```

**Without Docker:**
```bash
# Stop the application
wash app undeploy http-kv-counter

# Stop wasmCloud host
wash down

# Stop NATS server
# (Ctrl+C in the terminal running nats-server)
```

## CI/CD

The Docker-based setup is perfect for CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run tests
  run: |
    make docker-up
    make docker-deploy
    make docker-test
    make docker-down
```

## License

MIT
