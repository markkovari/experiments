# E2E Tests for wasmCloud Rate Limiters

This directory contains end-to-end tests for the rate limiter components deployed to wasmCloud with NATS KV storage.

## Prerequisites

1. **wasmCloud** - Install wasmCloud and wash CLI:
   ```bash
   curl -s https://wasmcloud.com/install.sh | bash
   ```

2. **NATS Server** - wasmCloud requires NATS:
   ```bash
   # Using Docker
   docker run -d --name nats -p 4222:4222 -p 6222:6222 -p 8222:8222 nats:latest -js

   # Or install NATS server locally
   # https://docs.nats.io/running-a-nats-service/introduction/installation
   ```

3. **Build components**:
   ```bash
   # From workspace root
   ./scripts/build-components.sh
   ```

## Running Tests

### Start wasmCloud Host

```bash
# In one terminal
wash up
```

This starts a local wasmCloud host with NATS embedded.

### Run E2E Tests

```bash
# From workspace root
cargo test --package e2e-tests -- --ignored --nocapture

# Or run specific test
cargo test --package e2e-tests test_token_bucket_e2e -- --ignored --nocapture
```

## Test Coverage

- **test_token_bucket_e2e** - Tests token bucket algorithm with refill
- **test_leaky_bucket_e2e** - Tests leaky bucket with constant leak rate
- **test_sliding_window_e2e** - Tests sliding window with expiry
- **test_multi_user_isolation** - Verifies per-user rate limiting
- **test_persistence_across_restarts** - Tests NATS KV persistence

## Manual Testing

You can also manually deploy and test:

```bash
# Deploy an app
wash app deploy wadm/token-bucket.yaml

# Check status
wash app list

# Invoke the component
wash call token-bucket-ratelimiter wasmcloud:ratelimit/rate-limiter init \
  --data '{"capacity":10,"refill_rate":1,"window_size_ms":0}'

wash call token-bucket-ratelimiter wasmcloud:ratelimit/rate-limiter check-rate-limit \
  --data '{"user_id":"user1","tokens_requested":5,"timestamp_ms":1000}'

# Undeploy
wash app undeploy token-bucket-ratelimiter
```

## NATS KV Inspection

To inspect the NATS KV buckets:

```bash
# List KV buckets
nats kv ls

# Get keys in bucket
nats kv ls ratelimit-token-bucket

# Get value
nats kv get ratelimit-token-bucket user1
```

## Architecture

```
┌─────────────────┐
│  E2E Test       │
│  (Rust)         │
└────────┬────────┘
         │ wash call
         ▼
┌─────────────────┐
│  wasmCloud Host │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐  ┌──────────────┐
│ Rate   │  │ NATS KV      │
│ Limiter│◄─┤ Capability   │
│ Comp.  │  │ Provider     │
└────────┘  └───────┬──────┘
                    │
                    ▼
            ┌──────────────┐
            │  NATS Server │
            │  (JetStream) │
            └──────────────┘
```

## Troubleshooting

- **Component not found**: Ensure you've built the components with `./scripts/build-components.sh`
- **Connection refused**: Check that NATS and wasmCloud host are running
- **Deployment failed**: Check logs with `wash app status <app-name>`
- **Tests timeout**: Increase sleep duration in tests or check component logs
