# Distributed Factorial Calculator

A distributed factorial calculator using Windmill, NATS, SurrealDB, and Go workers with a React frontend.

## Architecture

```
┌─────────────┐
│   React     │
│  Frontend   │
└──────┬──────┘
       │ HTTP
       ▼
┌─────────────┐      factorial.api.request      ┌──────────────────┐
│  Go API     │──────────────────────────────────▶│  Windmill        │
│  Service    │                                   │  Orchestrator    │
│             │◀──────────────────────────────────│  (Go NATS Sub)   │
└─────────────┘      factorial.api.response      └────────┬─────────┘
                                                           │
                                                           │ factorial.request
                     ┌─────────────┐                       │
                     │    NATS     │◀──────────────────────┘
                     │   Broker    │
                     └──────┬──────┘
                            │ factorial.response
                     ┌──────┴──────┐
                     │             │
              ┌──────▼─────┐ ┌────▼───────┐ ┌────▼───────┐
              │ Go Worker  │ │ Go Worker  │ │ Go Worker  │
              │     #1     │ │    #2      │ │    #3      │
              └──────┬─────┘ └────┬───────┘ └────┬───────┘
                     │            │              │
                     └──────┬─────┴──────────────┘
                            ▼
              ┌──────────────────┐     ┌─────────────┐
              │   NATS KV Cache  │     │  SurrealDB  │
              │ (Native TTL)     │     │   Logging   │
              └──────────────────┘     └─────────────┘
```

## Features

- **Distributed Computing**: Multiple Go workers process factorial calculations in parallel
- **Recursive Calculation**: Factorial computed recursively with distributed sub-tasks
- **Smart Caching**: Results cached in NATS KV with native TTL (auto-deletion)
- **Calculation Logging**: All operations logged to SurrealDB for analytics
- **Real-time Messaging**: NATS enables efficient pub/sub communication
- **Workflow Orchestration**: Windmill manages the calculation workflow
- **Modern UI**: React frontend with clean, responsive design

## Tech Stack

- **Windmill**: Workflow orchestration (Go scripts)
- **NATS**: Message broker for distributed communication + KV cache with TTL
- **SurrealDB**: Database for calculation logging
- **Go**: Backend workers and Windmill scripts
- **React**: Frontend UI
- **Docker Compose**: Service orchestration

## Project Structure

```
windmill-playground/
├── docker-compose.yml          # All services orchestration
├── backend/                    # Go services
│   ├── cmd/
│   │   ├── worker/main.go     # Factorial worker
│   │   └── api/main.go        # HTTP API service
│   ├── internal/
│   │   ├── cache/             # SurrealDB client
│   │   ├── calculator/        # Factorial logic
│   │   └── messaging/         # NATS client
│   ├── Dockerfile             # Worker Dockerfile
│   ├── Dockerfile.api         # API Dockerfile
│   └── go.mod
├── windmill-scripts/           # Windmill orchestrator
│   ├── nats_subscriber.go     # NATS event orchestrator
│   ├── Dockerfile
│   └── go.mod
├── frontend/                   # React app
│   ├── src/
│   │   └── components/Calculator.jsx
│   ├── Dockerfile
│   └── package.json
└── README.md
```

## How It Works

1. **User enters a number** in the React UI
2. **Frontend** sends HTTP POST to Go API service (`/calculate`)
3. **API service** publishes request to NATS topic `factorial.api.request`
4. **Windmill Orchestrator** (Go NATS subscriber) receives the request
5. **Orchestrator** forwards request to workers via `factorial.request` topic
6. **3 Go Workers** receive the request (all 3 process in parallel)
7. **Workers** check NATS KV cache for cached results
8. If not cached, workers **recursively calculate**:
   - n! = n × (n-1)!
   - Each sub-calculation is also cached
9. **Results cached** in NATS KV and all calculations logged to SurrealDB
10. **Workers** publish responses to `factorial.response` topic
11. **Orchestrator** receives worker responses
12. **Orchestrator** forwards first response to `factorial.api.response` topic
13. **API service** receives response and returns to frontend
14. **Frontend** displays the result

## Getting Started

### Prerequisites

- Docker and Docker Compose
- Go 1.21+ (for local development)
- Node.js 18+ (for local development)

### Running with Docker Compose

1. Clone the repository:
```bash
git clone <repository-url>
cd windmill-playground
```

2. Start all services:
```bash
docker-compose up --build
```

3. Wait for services to be ready (~30 seconds)

4. Access the services:
   - **Frontend**: http://localhost:3000
   - **API**: http://localhost:8080
   - **NATS Monitor**: http://localhost:8222
   - **SurrealDB**: http://localhost:8000
   - **Windmill UI** (optional): http://localhost:8001

### Using the Calculator

1. Open http://localhost:3000 in your browser

2. Enter a number (e.g., 10)

3. Click "Calculate Factorial"

4. View the result, request ID, and see the distributed calculation in action

### Monitoring

#### NATS Messages

```bash
# Subscribe to all messages
docker exec -it <nats-container> nats sub ">"

# Subscribe to factorial requests
docker exec -it <nats-container> nats sub "factorial.request"

# Subscribe to factorial responses
docker exec -it <nats-container> nats sub "factorial.response"
```

#### NATS KV Cache

```bash
# View NATS KV bucket info (requires nats CLI in container)
docker exec -it <nats-container> nats kv info factorial_cache

# List all cached keys
docker exec -it <nats-container> nats kv ls factorial_cache
```

#### SurrealDB Logs

```bash
# Query calculation logs
curl -X POST http://localhost:8000/sql \
  -H "Content-Type: application/json" \
  -u "root:root" \
  -d '{"sql":"USE NS factorial DB calculations; SELECT * FROM calculation_log ORDER BY calculated_at DESC;"}'
```

#### Worker Logs

```bash
# View all worker logs
docker-compose logs -f factorial-worker
```

## Development

### Backend Worker

```bash
cd backend
go mod download
go run cmd/worker/main.go
```

Environment variables:
- `NATS_URL`: NATS server URL (default: `nats://localhost:4222`)
- `SURREALDB_URL`: SurrealDB WebSocket URL (default: `ws://localhost:8000/rpc`)
- `SURREALDB_USER`: SurrealDB user (default: `root`)
- `SURREALDB_PASS`: SurrealDB password (default: `root`)
- `SURREALDB_NS`: SurrealDB namespace (default: `factorial`)
- `SURREALDB_DB`: SurrealDB database (default: `calculations`)
- `CACHE_TTL`: Cache expiration time (default: `24h`, examples: `1h`, `30m`, `2h30m`)
- `LOG_LEVEL`: Log level - `debug`, `info`, `warn`, `error` (default: `info`)
- `LOG_JSON`: Enable JSON logging - `true` or `false` (default: `false`)

Command-line flags:
```bash
# Set log level
./worker --log-level debug

# Enable JSON logging
./worker --log-json

# Both
./worker --log-level debug --log-json
```

### Frontend

```bash
cd frontend
npm install
npm start
```

Environment variables:
- `REACT_APP_WINDMILL_URL`: Windmill API URL (default: `http://localhost:8001`)

## Example Calculation Flow

For `factorial(5)`:

1. Initial request: Calculate 5!
2. Worker checks cache → miss
3. Recursive calls:
   - Calculate 4! → checks cache → miss
   - Calculate 3! → checks cache → miss
   - Calculate 2! → checks cache → miss
   - Calculate 1! → checks cache → miss → returns 1 → **cache**
   - Product: 2 × 1 = 2 → **cache** → returns 2!
   - Product: 3 × 2 = 6 → **cache** → returns 3!
   - Product: 4 × 6 = 24 → **cache** → returns 4!
   - Product: 5 × 24 = 120 → **cache** → returns 5!
4. Result: 120
5. All calculations logged in SurrealDB

If you calculate `factorial(6)` next:
- 1!, 2!, 3!, 4!, 5! are **cache hits**
- Only 6! needs to be calculated
- Much faster response!

### Cache TTL

Cache entries automatically expire after the configured `CACHE_TTL` (default: 24 hours):
- NATS KV has **native TTL support** - expired entries are automatically deleted by NATS
- No manual cleanup needed (unlike SurrealDB)
- Efficient memory management with automatic eviction
- Prevents stale data accumulation

## Logging

### Log Levels

- **debug**: Verbose logging including cache hits/misses, message routing
- **info**: Standard operational messages (default)
- **warn**: Warning messages for recoverable issues
- **error**: Error messages for failures

### Log Formats

**Standard format** (default):
```
[INFO] [worker-abc123] Received factorial request number=10 request_id=xyz
[DEBUG] [worker-abc123] Cache hit for factorial number=10 result=3628800 duration=2
```

**JSON format** (`LOG_JSON=true`):
```json
{"level":"info","component":"worker-abc123","msg":"Received factorial request","timestamp":"2026-01-25T00:00:00Z","fields":{"number":10,"request_id":"xyz"}}
```

### Configuration

Set via environment variables:
```yaml
environment:
  LOG_LEVEL: debug
  LOG_JSON: "true"
  CACHE_TTL: 2h
```

Or via command-line flags:
```bash
./worker --log-level debug --log-json
./api -log-level info
```

## Performance Considerations

- **Caching**: Dramatically speeds up subsequent calculations
- **Distributed Workers**: Multiple workers process calculations in parallel
- **NATS**: Low-latency message delivery + in-memory KV cache
- **Native TTL**: Automatic cache eviction without manual cleanup overhead
- **SurrealDB**: Persistent calculation logs for analytics

## Scaling

Scale workers horizontally:

```bash
docker-compose up --scale factorial-worker=5
```

## Cleanup

```bash
docker-compose down -v
```

## License

MIT
