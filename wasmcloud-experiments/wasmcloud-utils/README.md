# wasmCloud Rate Limiters

Three production-ready rate limiting algorithms implemented as wasmCloud components with NATS KV persistence.

## 🚀 Components

### 1. Token Bucket
Allows burst traffic up to capacity, with tokens refilling at a constant rate.

**Use cases:** API rate limiting, traffic shaping with bursts

```
Capacity: 10 tokens
Refill: 1 token/sec
Allows: Burst of 10 requests, then 1/sec
```

### 2. Leaky Bucket
Processes requests at a constant rate with a queue (leak rate).

**Use cases:** Smooth traffic flow, preventing thundering herd

```
Capacity: 10 requests
Leak: 2 requests/sec
Queues: Excess requests leak out steadily
```

### 3. Sliding Window
Tracks requests within a rolling time window.

**Use cases:** Hourly/daily limits, precise time-based quotas

```
Capacity: 100 requests
Window: 3600000ms (1 hour)
Tracks: Requests in last hour
```

## 📦 Architecture

```
┌─────────────────────────────────────────────┐
│         wasmCloud Application               │
│                                             │
│  ┌──────────────┐      ┌─────────────────┐│
│  │ Rate Limiter │◄────►│  NATS KV        ││
│  │  Component   │      │  (Persistence)  ││
│  └──────────────┘      └─────────────────┘│
└─────────────────────────────────────────────┘
          ▲
          │ WIT Interface
          │
    wasmcloud:ratelimit
```

## 🛠️ Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasmCloud
curl -s https://wasmcloud.com/install.sh | bash

# Start NATS (via Docker)
docker run -d --name nats -p 4222:4222 -p 6222:6222 -p 8222:8222 nats:latest -js
```

### Build & Test

```bash
# Run unit tests
cargo test --workspace

# Build wasmCloud components
./scripts/build-components.sh

# Start wasmCloud
wash up

# Run E2E tests (in another terminal)
cargo test --package e2e-tests -- --ignored --nocapture
```

## 📖 Usage

### Deploy to wasmCloud

```bash
# Deploy token bucket
wash app deploy wadm/token-bucket.yaml

# Check status
wash app list
wash app status token-bucket-ratelimiter
```

### Invoke via wash CLI

```bash
# Initialize rate limiter
wash call token-bucket-ratelimiter wasmcloud:ratelimit/rate-limiter init \
  --data '{"capacity":10,"refill_rate":1,"window_size_ms":0}'

# Check rate limit
wash call token-bucket-ratelimiter wasmcloud:ratelimit/rate-limiter check-rate-limit \
  --data '{"user_id":"user1","tokens_requested":5,"timestamp_ms":1000}'

# Reset user
wash call token-bucket-ratelimiter wasmcloud:ratelimit/rate-limiter reset \
  --data '"user1"'
```

### Response Format

```json
{
  "allowed": true,
  "tokens_remaining": 5,
  "retry_after_ms": null
}
```

## 🧪 Testing

### Unit Tests
```bash
cargo test --workspace
```

**Coverage:**
- Token bucket: Basic operations, refill logic
- Leaky bucket: Queue management, leak rate
- Sliding window: Window expiry, partial expiry

### E2E Tests
```bash
cargo test --package e2e-tests -- --ignored
```

**Coverage:**
- Deployment and invocation
- Multi-user isolation
- NATS KV persistence
- Cross-restart state

See [tests/e2e/README.md](tests/e2e/README.md) for details.

## 📁 Project Structure

```
├── wit/
│   ├── rate-limiter.wit          # Common WIT interface
│   └── deps/keyvalue/            # WASI keyvalue imports
├── token-bucket/                 # Token bucket component
├── leaky-bucket/                 # Leaky bucket component
├── sliding-window/               # Sliding window component
├── wadm/                         # wasmCloud app manifests
│   ├── token-bucket.yaml
│   ├── leaky-bucket.yaml
│   └── sliding-window.yaml
├── tests/e2e/                    # End-to-end tests
└── scripts/
    └── build-components.sh       # Build script
```

## 🔌 WIT Interface

```wit
interface rate-limiter {
    init: func(config: rate-limit-config) -> result<_, rate-limit-error>;
    check-rate-limit: func(request: rate-limit-request) -> result<rate-limit-response, rate-limit-error>;
    reset: func(user-id: string) -> result<_, rate-limit-error>;
}
```

**Configuration:**
- `capacity`: Maximum tokens/requests
- `refill-rate`: Tokens per second (token/leaky bucket)
- `window-size-ms`: Time window in ms (sliding window)

## 🔍 Monitoring

### NATS KV Inspection

```bash
# List buckets
nats kv ls

# View keys
nats kv ls ratelimit-token-bucket

# Get user state
nats kv get ratelimit-token-bucket user1
```

### wasmCloud Logs

```bash
# Component logs
wash app logs token-bucket-ratelimiter

# Host logs
wash logs
```

## 🚢 Production Deployment

### Distributed wasmCloud

```yaml
# wadm/production.yaml
apiVersion: core.oam.dev/v1beta1
kind: Application
metadata:
  name: token-bucket-ratelimiter
spec:
  components:
    - name: token-bucket
      type: component
      properties:
        image: ghcr.io/your-org/token-bucket:v0.1.0
      traits:
        - type: spreadscaler
          properties:
            replicas: 3  # HA deployment
            spread:
              - name: multi-region
                requirements:
                  zone: ["us-east-1", "us-west-2", "eu-west-1"]
```

### NATS JetStream Cluster

For production, use a NATS cluster with JetStream for durability:

```bash
# Configure NATS cluster
# https://docs.nats.io/running-a-nats-service/configuration/clustering
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test --workspace`
5. Run e2e tests: `cargo test --package e2e-tests -- --ignored`
6. Submit a pull request

## 📄 License

MIT

## 🔗 Resources

- [wasmCloud Documentation](https://wasmcloud.com/docs)
- [NATS Documentation](https://docs.nats.io)
- [Component Model](https://component-model.bytecodealliance.org)
- [Rate Limiting Algorithms](https://en.wikipedia.org/wiki/Rate_limiting)

---

## Workflow API

A stateful HTTP workflow engine running as a wasmCloud WASM component. It stores all state in NATS JetStream KV and exposes a REST API for defining, running, and monitoring multi-step workflows with retry, branching, sub-workflow delegation, and event subscription.

### Quick Start

```bash
# 1. Register a workflow
curl -s -X POST http://localhost:8080/workflows \
  -H 'content-type: application/json' \
  -d '{"name":"simple-job","steps":[{"name":"run","depends_on":[]}]}'

# 2. Start a run
curl -s -X POST http://localhost:8080/runs \
  -H 'content-type: application/json' \
  -d '{"wf_name":"simple-job"}'

# 3. Poll run status (replace RUN_ID with the id from step 2)
curl -s http://localhost:8080/runs/RUN_ID
```

Full API reference: [docs/workflow-api/API.md](docs/workflow-api/API.md)

Architecture overview: [docs/workflow-api/ARCHITECTURE.md](docs/workflow-api/ARCHITECTURE.md)

### Running Cucumber (BDD) Tests

The cucumber test suite exercises the live API against a deployed wasmCloud host.

```bash
# Start wasmCloud and NATS
wash up -d

# Build and deploy the workflow-api component
wash build -p workflow-api
wash app deploy wadm/workflow-api.yaml

# Run the BDD tests
cargo test -p workflow-api-cucumber
```

Feature files are in `tests/cucumber/features/`. Step definitions are in `tests/cucumber/tests/steps/`.
