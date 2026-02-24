# HTTP KV Counter - wasmCloud Component

A wasmCloud component written in Rust that provides an HTTP API for managing counters stored in a NATS KV store with a 3-second TTL.

## Features

- **HTTP API** for counter management
  - `GET /` - Returns info message
  - `GET /:name` - Returns a specific counter value
  - `POST /:name` - Increments counter (creates with value 1 if not exists)
- **NATS KV Store** with automatic 3-second TTL expiration
- **Atomic operations** for thread-safe counter increments
- **Horizontal scaling** - Deployed with 10 component instances for 5-10k req/sec throughput
- **Comprehensive testing** - 26 tests covering unit, integration, E2E, and stress scenarios
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
cargo test --test e2e_test -- --ignored --test-threads=1
```

#### Stress Tests

Performance and load tests with 8 comprehensive scenarios:

```bash
# macOS (aarch64)
cargo test --test stress_test --target aarch64-apple-darwin -- --ignored --test-threads=1

# Linux (x86_64)
cargo test --test stress_test --target x86_64-unknown-linux-gnu -- --ignored --test-threads=1
```

**Test scenarios include:**
1. **Sequential requests** - 1,000 sequential counter increments
2. **High concurrency** - 100 parallel counter operations
3. **Many unique counters** - 500 different counter names
4. **Mixed workload** - 10 counters × 50 increments each
5. **Sustained load** - 50 req/sec for 30 seconds
6. **TTL stress** - 100 counters expiring after 3 seconds
7. **Burst traffic** - 200 simultaneous requests
8. **Read-heavy workload** - 1,000 reads + 10 writes

**Note:** Run stress tests with `--test-threads=1` to avoid resource contention and get accurate performance metrics.

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

### Get Service Info

```bash
curl http://localhost:8080/
# Response: {"message": "Counter service. Use POST /:name to increment, GET /:name to read."}
```

### TTL Behavior

After 3 seconds of inactivity, counters are automatically deleted:

```bash
curl -X POST http://localhost:8080/temp
# Response: {"name":"temp","value":1}

sleep 4

curl http://localhost:8080/temp
# Response: {"name":"temp","value":0}
```

## Performance & Scaling

### Horizontal Scaling

The application is configured to run with **10 component instances** for improved throughput and load distribution:

```yaml
# In wadm.yaml
traits:
  - type: spreadscaler
    properties:
      instances: 10
```

To modify the number of instances:
1. Edit `wadm.yaml` and change `instances: 10` to your desired value
2. Redeploy: `wash app deploy wadm.yaml`
3. Verify: `wash app status http-kv-counter`

### Performance Characteristics

**Single Instance:**
- ~1,500 req/sec average throughput
- Suitable for low-traffic applications

**10 Instances (current configuration):**
- ~5,000-10,000 req/sec throughput
- Better load distribution across instances
- Improved availability and fault tolerance

**NATS KV Performance Notes:**
- NATS KV prioritizes **consistency and durability** over raw speed
- Throughput is limited by NATS JetStream architecture (not the component)
- For comparison: Redis KV could achieve 50k+ req/sec but lacks native TTL integration
- Trade-off: Native TTL support and wasmCloud integration vs maximum throughput

### Benchmarking

Run stress tests to measure performance in your environment:

```bash
# Sustained load test (50 req/sec for 30 seconds)
cargo test --test stress_test test_sustained_load --target aarch64-apple-darwin -- --ignored --nocapture

# Burst traffic test (200 simultaneous requests)
cargo test --test stress_test test_burst_traffic --target aarch64-apple-darwin -- --ignored --nocapture

# High concurrency test (100 parallel operations)
cargo test --test stress_test test_high_concurrency --target aarch64-apple-darwin -- --ignored --nocapture
```

## Project Structure

```
.
├── Cargo.toml              # Rust dependencies (wasmcloud-component, test deps)
├── wasmcloud.toml          # wasmCloud component config
├── wadm.yaml               # Deployment manifest (10 instances configured)
├── wit/                    # WebAssembly Interface definitions
│   ├── world.wit           # Component interface definition
│   └── deps.toml           # WIT dependencies
├── src/
│   └── lib.rs              # Main component implementation (routing + KV ops)
└── tests/
    ├── integration_test.rs # Integration tests (9 tests)
    ├── e2e_test.rs         # End-to-end tests (9 tests)
    └── stress_test.rs      # Stress/performance tests (8 tests)
```

**Test Coverage:**
- **9 integration tests** - Business logic, path parsing, JSON serialization
- **9 E2E tests** - Full runtime testing with NATS, TTL verification, concurrency
- **8 stress tests** - Performance benchmarking, load testing, burst scenarios

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
target_config:
  - name: counters-config
    properties:
      bucket: counters
      max_age: 3s
```

This configuration ensures counters automatically expire 3 seconds after their last update.

### Scaling Configuration

The application runs with 10 component instances by default:

```yaml
- name: http-kv-counter
  type: component
  properties:
    image: file://./build/http_kv_counter_s.wasm
  traits:
    - type: spreadscaler
      properties:
        instances: 10
```

To change the number of instances:
1. Edit `wadm.yaml`
2. Modify `instances: 10` to your desired value
3. Redeploy: `wash app deploy wadm.yaml`

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
