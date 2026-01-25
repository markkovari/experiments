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
- **Distributed Recursion**: Each recursive step (n-1)! is distributed across workers via NATS
- **Smart Caching**: Results cached in NATS KV with native TTL (auto-deletion)
- **Calculation Logging**: All operations logged to SurrealDB for analytics
- **Real-time Messaging**: NATS enables efficient pub/sub and request-response communication
- **Workflow Orchestration**: NATS orchestrates the distributed recursive workflow
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
6. **Worker A** receives factorial(10) request
7. **Worker A** checks NATS KV cache - miss
8. **Worker A** makes NATS request for factorial(9) (distributed recursion!)
9. **Worker B** receives factorial(9) request, checks cache - miss
10. **Worker B** makes NATS request for factorial(8)
11. **Worker C** receives factorial(8) request... (recursion continues)
12. Eventually a worker finds factorial(1) in cache or calculates it
13. **Results bubble back** through NATS request-response chain
14. Each worker calculates its product (n × prev) and caches in NATS KV
15. All calculations logged to SurrealDB
16. **Worker A** receives final result, publishes to `factorial.response`
17. **Orchestrator** forwards to `factorial.api.response`
18. **API service** receives response and returns to frontend
19. **Frontend** displays the result

**Key insight**: Each recursive step can be handled by a different worker, truly distributing the computation!

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
   - **Windmill UI**: http://localhost:8001

### Two Orchestration Modes

The system supports two modes:

#### Direct NATS Mode (Default)
- Faster, lower latency
- Distributed recursion via NATS request-response
- No job tracking UI
- ✅ Works out of the box

#### Windmill Orchestration Mode
- Full job observability in Windmill UI
- View all runs at http://localhost:8001/runs
- Track execution history
- Slightly higher latency due to job scheduling
- 📋 **Requires setup** - see [WINDMILL_SETUP.md](WINDMILL_SETUP.md)

### Using the Calculator

1. Open http://localhost:3000 in your browser

2. Enter a number (e.g., 10)

3. Click "Calculate Factorial"

4. View the result and request ID

**To view job executions in Windmill UI:**
- See [WINDMILL_SETUP.md](WINDMILL_SETUP.md) for enabling Windmill mode
- Then access http://localhost:8001/runs to see all calculations

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

## Example Distributed Calculation Flow

For `factorial(5)` with 3 workers:

1. **Worker-1** receives: Calculate 5!
2. **Worker-1** checks NATS KV cache → miss
3. **Worker-1** → NATS request: "Need factorial(4)"
4. **Worker-2** receives: Calculate 4!
5. **Worker-2** checks cache → miss
6. **Worker-2** → NATS request: "Need factorial(3)"
7. **Worker-3** receives: Calculate 3!
8. **Worker-3** checks cache → miss
9. **Worker-3** → NATS request: "Need factorial(2)"
10. **Worker-1** receives: Calculate 2! (round-robin)
11. **Worker-1** checks cache → miss
12. **Worker-1** → NATS request: "Need factorial(1)"
13. **Worker-2** receives: Calculate 1!
14. **Worker-2** finds base case → returns 1 → **cache in NATS KV**
15. **Worker-1** receives "1" → calculates 2 × 1 = 2 → **cache** → returns 2!
16. **Worker-3** receives "2" → calculates 3 × 2 = 6 → **cache** → returns 3!
17. **Worker-2** receives "6" → calculates 4 × 6 = 24 → **cache** → returns 4!
18. **Worker-1** receives "24" → calculates 5 × 24 = 120 → **cache** → returns 5!
19. Result: **120** (calculated by multiple workers!)
20. All operations logged in SurrealDB

**Next request for factorial(6)**:
- 1!, 2!, 3!, 4!, 5! are **cache hits** in NATS KV
- Only one worker needed to calculate 6 × 120 = 720
- **Instant response!**

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
- **Distributed Recursion**: Each recursive step can be handled by different workers
- **Load Balancing**: NATS automatically distributes work across available workers
- **NATS Request-Response**: Synchronous RPC pattern with 30-second timeout
- **In-Memory KV**: Fast cache lookups with native TTL
- **Automatic Eviction**: No cleanup overhead - NATS handles TTL
- **SurrealDB**: Persistent calculation logs for analytics

**Note**: For small factorials, distributed recursion may add overhead. For large computations or high load, it enables true horizontal scaling.

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
