# Docker Test Harness Setup

This document describes the Docker-based test harness that allows you to develop and test the wasmCloud application **without installing NATS or wash locally**.

## What Changed

### New Files Added

1. **docker-compose.yml** - Main orchestration file
   - NATS server with JetStream
   - wasmCloud host
   - Local OCI registry
   - Health checks for all services

2. **Dockerfile** - Multi-stage build for the Rust component
   - Builder stage with wash CLI
   - Minimal runtime stage with built artifacts

3. **docker-compose.test.yml** - Test orchestration
   - Builder service for compiling
   - Deployer service for deployment
   - Test runner service for e2e tests

4. **scripts/wait-for-ready.sh** - Service health checker
   - Waits for NATS to be ready
   - Waits for JetStream to be ready
   - Waits for wasmCloud host to be ready

5. **scripts/deploy-app.sh** - Deployment automation
   - Builds component if needed
   - Deploys to wasmCloud
   - Waits for HTTP endpoint

6. **scripts/run-tests.sh** - Test runner
   - Runs unit, integration, and e2e tests
   - Color-coded output
   - Configurable test types

7. **.dockerignore** - Build optimization
   - Excludes unnecessary files from Docker context

### Modified Files

1. **Makefile** - Added Docker commands
   - `docker-up`, `docker-down` - Service management
   - `docker-build`, `docker-deploy` - Build and deployment
   - `docker-test`, `docker-test-e2e` - Testing
   - `docker-logs` - Log viewing
   - `docker-clean` - Cleanup

2. **README.md** - Added Docker documentation
   - Quick Start with Docker section
   - Docker-specific troubleshooting
   - Docker architecture diagram

## Quick Start

```bash
# 1. Start all services
make docker-up

# 2. Build and deploy
make docker-deploy

# 3. Test
curl -X POST http://localhost:8080/test
curl http://localhost:8080/test

# 4. Run tests
make docker-test-e2e

# 5. Clean up
make docker-down
```

## How It Works

### Service Startup

1. **NATS container** starts with JetStream enabled
   - Stores data in a Docker volume
   - Exposes ports 4222 (client), 6222 (cluster), 8222 (monitoring)

2. **wasmCloud host** starts and connects to NATS
   - Configured to use NATS for all communication
   - Allows insecure OCI registry access
   - Exposes port 4000 for API

3. **HTTP endpoint** is exposed via wasmCloud
   - Port 8080 is mapped to the host

### Build Process

```bash
make docker-build
```

1. Docker builds the Dockerfile
2. Installs wash CLI in the builder container
3. Compiles Rust code to wasm32-wasip2
4. Outputs to `build/` directory

### Deployment Process

```bash
make docker-deploy
```

1. Waits for services to be healthy
2. Runs builder to compile component
3. Runs deployer to deploy wadm.yaml
4. Waits for HTTP endpoint to respond

### Testing Process

```bash
make docker-test-e2e
```

1. Waits for services to be ready
2. Waits for HTTP endpoint
3. Runs e2e tests from `tests/e2e_test.rs`
4. Tests include TTL expiration verification

## Advantages of Docker Setup

### For Development
- ✅ No need to install NATS locally
- ✅ No need to install wash CLI locally
- ✅ Consistent environment across team
- ✅ Easy cleanup with `docker-down`
- ✅ Isolated from host system

### For Testing
- ✅ Automated test environment setup
- ✅ Reproducible test conditions
- ✅ TTL testing works reliably
- ✅ Easy to run in CI/CD
- ✅ Full e2e testing capabilities

### For CI/CD
- ✅ Single Docker Compose command
- ✅ No external dependencies
- ✅ Parallel test execution
- ✅ Automatic cleanup
- ✅ Logs available for debugging

## Architecture

```
┌─────────────────────────────────────────────────┐
│             Docker Compose Network              │
│                                                 │
│  ┌──────────────┐       ┌────────────────────┐ │
│  │ NATS:2.10    │◄──────┤ wasmCloud Host     │ │
│  │ JetStream    │       │ + HTTP Provider    │ │
│  │ Port 4222    │       │ + KV NATS Provider │ │
│  │ Volume: data │       │ + Component        │ │
│  └──────────────┘       └─────────┬──────────┘ │
│                                   │            │
│  ┌──────────────┐                 │            │
│  │ OCI Registry │                 │            │
│  │ Port 5001    │                 │            │
│  └──────────────┘                 │            │
│                                   │            │
│                         ┌─────────▼─────────┐  │
│                         │   HTTP :8080      │  │
│                         └───────────────────┘  │
└─────────────────────────────────┬───────────────┘
                                  │
                        ┌─────────▼──────────┐
                        │  Host: localhost   │
                        │  curl localhost:8080│
                        └────────────────────┘
```

## Common Commands

```bash
# Service Management
make docker-up          # Start services
make docker-down        # Stop services
make docker-restart     # Restart everything
make docker-ps          # Show service status

# Development
make docker-build       # Build component
make docker-deploy      # Deploy to wasmCloud
make docker-run         # Start + deploy (all-in-one)

# Testing
make docker-test        # Run all tests
make docker-test-e2e    # Run e2e tests only
make docker-test-all    # Full test suite

# Debugging
make docker-logs                # All logs
make docker-logs-wasmcloud      # wasmCloud only
make docker-logs-nats           # NATS only
make docker-shell-wasmcloud     # Shell into wasmCloud
make docker-shell-nats          # Shell into NATS

# Cleanup
make docker-clean       # Remove everything
```

## Troubleshooting

### Services won't start
```bash
docker compose ps
docker compose logs
```

### Port conflicts
Edit `docker-compose.yml` and change port mappings:
```yaml
ports:
  - "8081:8080"  # Change host port
```

### Build failures
```bash
# Clean and rebuild
make docker-clean
make docker-build
```

### Test failures
```bash
# Check logs
make docker-logs-wasmcloud

# Verify services
docker compose ps

# Try restarting
make docker-restart
```

## Next Steps

1. Try the Quick Start above
2. Make changes to `src/lib.rs`
3. Rebuild with `make docker-build`
4. Redeploy with `make docker-deploy`
5. Test with `make docker-test-e2e`

## Comparison: Docker vs Local

| Feature | Docker | Local |
|---------|--------|-------|
| Setup time | ~2 min | ~15-30 min |
| Dependencies | Docker only | Rust + wash + NATS |
| Isolation | Full | None |
| Cleanup | Single command | Multiple steps |
| CI/CD ready | Yes | No |
| Hot reload | No* | Yes (wash dev) |
| Performance | ~10% slower | Native |

*Can be added with volume mounts and watch scripts

## Tips

1. **Use docker-run for first-time setup**
   ```bash
   make docker-run
   ```

2. **Keep logs open while developing**
   ```bash
   make docker-logs &
   ```

3. **Use docker-clean between major changes**
   ```bash
   make docker-clean
   make docker-run
   ```

4. **Check service health before testing**
   ```bash
   docker compose ps
   # All services should show "healthy"
   ```

5. **Volume cleanup for fresh start**
   ```bash
   docker compose down -v
   ```
