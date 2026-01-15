# Veterinary Clinic Backend - Rust Monorepo with SurrealDB

A modern, type-safe backend system for managing a veterinary clinic with embedded SurrealDB, built as a Rust monorepo.

## Features

- **Domain Models**: Users, Pets, Doctors, and Health Checks
- **Embedded SurrealDB**: High-performance, schema-enforced database with RocksDB storage
- **REST API**: Built with Axum web framework
- **Database Migrations**: Schema versioning and seed data management
- **Comprehensive Testing**: Unit, integration, and E2E tests with nextest
- **Type Safety**: Full compile-time guarantees across the stack

## Project Structure

```
surreal-backend/
├── crates/
│   ├── core/          # Domain models and business logic
│   ├── db/            # SurrealDB integration and repositories
│   ├── migrations/    # Database migrations and seed data
│   ├── api/           # REST API with Axum
│   └── cli/           # Main binary and CLI commands
├── tests/
│   ├── integration/   # Cross-crate integration tests
│   └── e2e/           # End-to-end API tests with Docker
└── .config/
    └── nextest.toml   # Nextest configuration
```

## Getting Started

### Prerequisites

- Rust 1.75+ (edition 2021)
- Docker and Docker Compose (for containerized deployment)
- cargo-nextest (optional, for parallel testing)

### Quick Start with Docker Compose

The easiest way to run the entire stack:

```bash
# Start SurrealDB and the API
docker-compose up -d

# View logs
docker-compose logs -f api

# Stop services
docker-compose down
```

The API will be available at `http://localhost:3000` and SurrealDB at `http://localhost:8000`.

### Local Development

#### Option 1: With Docker (Recommended)

```bash
# Start only SurrealDB
docker-compose up surrealdb -d

# Copy environment file
cp .env.example .env

# Edit .env to use remote database
# DATABASE_URL=http://localhost:8000
# DATABASE_USERNAME=root
# DATABASE_PASSWORD=root

# Run the API locally
cargo run --bin surreal-cli serve --seed
```

#### Option 2: Fully Local (Embedded RocksDB)

```bash
# Copy environment file
cp .env.example .env

# Use embedded RocksDB (default)
# DATABASE_URL=rocksdb://./data/surrealdb

# Build the project
cargo build --release

# Run with migrations and seed data
cargo run --bin surreal-cli serve --seed
```

### Running the Server

```bash
# Start server with migrations and seed data
cargo run --bin surreal-backend -- serve --migrate --seed

# Start server on custom port
cargo run --bin surreal-backend -- serve --port 8080

# Start server with custom database path
cargo run --bin surreal-backend -- serve --db-path ./my-data
```

### Database Management

```bash
# Run migrations only
cargo run --bin surreal-backend -- migrate

# Run migrations with seed data
cargo run --bin surreal-backend -- migrate --seed
```

## Testing

### Run All Tests with Nextest

```bash
# Run all tests in parallel
cargo nextest run

# Run with CI profile
cargo nextest run --profile ci
```

### Run Specific Test Suites

```bash
# Unit tests only (fast)
cargo nextest run --profile unit

# Integration tests
cargo nextest run --profile integration

# E2E tests (requires Docker)
cargo nextest run --profile e2e
```

### Traditional Cargo Test

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p surreal-core
cargo test -p surreal-db
```

## API Documentation

### Base URL

`http://localhost:3000`

### Endpoints

#### Health Check
- `GET /health` - API health status

#### Users
- `POST /users` - Create user
- `GET /users` - List all users
- `GET /users/:id` - Get user by ID
- `PUT /users/:id` - Update user
- `DELETE /users/:id` - Delete user

#### Pets
- `POST /pets` - Create pet
- `GET /pets` - List all pets
- `GET /pets/:id` - Get pet by ID
- `GET /users/:owner_id/pets` - Get pets by owner
- `PUT /pets/:id` - Update pet
- `DELETE /pets/:id` - Delete pet

#### Doctors
- `POST /doctors` - Create doctor
- `GET /doctors` - List all doctors
- `GET /doctors/available` - List available doctors
- `GET /doctors/:id` - Get doctor by ID
- `PUT /doctors/:id` - Update doctor
- `DELETE /doctors/:id` - Delete doctor

#### Health Checks
- `POST /checks` - Schedule health check
- `GET /checks` - List all checks
- `GET /checks/:id` - Get check by ID
- `GET /pets/:pet_id/checks` - Get checks by pet
- `GET /doctors/:doctor_id/checks` - Get checks by doctor
- `PATCH /checks/:id/start` - Start a check
- `PATCH /checks/:id/complete` - Complete a check
- `PATCH /checks/:id/cancel` - Cancel a check
- `DELETE /checks/:id` - Delete check

### Example Requests

#### Create User
```bash
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "name": "John Doe",
    "phone": "+1234567890",
    "address": "123 Main St"
  }'
```

#### Create Pet
```bash
curl -X POST http://localhost:3000/pets \
  -H "Content-Type: application/json" \
  -d '{
    "owner_id": "<user-uuid>",
    "name": "Buddy",
    "species": "Dog",
    "breed": "Golden Retriever",
    "weight_kg": 30.5
  }'
```

#### Schedule Health Check
```bash
curl -X POST http://localhost:3000/checks \
  -H "Content-Type: application/json" \
  -d '{
    "pet_id": "<pet-uuid>",
    "doctor_id": "<doctor-uuid>",
    "scheduled_at": "2026-01-20T10:00:00Z"
  }'
```

## Configuration

### Environment Variables

The application follows [12-Factor App](https://12factor.net/) principles, with all configuration via environment variables:

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `DATABASE_URL` | Database connection URL | `rocksdb://./data/surrealdb` | `http://surrealdb:8000` |
| `DATABASE_USERNAME` | Database username (remote only) | `root` | `admin` |
| `DATABASE_PASSWORD` | Database password (remote only) | `root` | `secret` |
| `PORT` | Server port | `3000` | `8080` |
| `RUST_LOG` | Logging level | `info` | `debug` |

### Database URL Formats

- **Embedded RocksDB**: `rocksdb://./path/to/data`
- **In-memory**: `mem://`
- **Remote HTTP**: `http://localhost:8000`
- **Remote WebSocket**: `ws://localhost:8000`

### 12-Factor App Compliance

This application follows the [12-Factor App](https://12factor.net/) methodology:

- **III. Config**: All configuration via environment variables
- **IV. Backing Services**: Database treated as attached resource
- **VI. Processes**: Stateless app, state stored in database
- **VIII. Concurrency**: Horizontally scalable via containers
- **IX. Disposability**: Fast startup, graceful shutdown
- **XI. Logs**: Logs to stdout/stderr for aggregation

## Development

### Code Organization

- **`crates/core`**: Domain models with validation and business logic
- **`crates/db`**: Repository pattern with SurrealDB integration
- **`crates/migrations`**: Schema definitions and seed data
- **`crates/api`**: REST API handlers and routes
- **`crates/cli`**: Application entry point and CLI commands

### Adding New Features

1. Define domain models in `crates/core/src/models/`
2. Create repository in `crates/db/src/repository/`
3. Add schema definition in `crates/migrations/src/schema.rs`
4. Implement API handlers in `crates/api/src/handlers/`
5. Add routes in `crates/api/src/routes.rs`
6. Write tests at all levels

### Running Development Server

```bash
# Watch mode with cargo-watch
cargo install cargo-watch
cargo watch -x 'run --bin surreal-backend -- serve --migrate --seed'
```

## Architecture Highlights

### Domain-Driven Design
- Core domain models with rich behavior
- Repository pattern for data access
- Clear separation of concerns

### Type Safety
- Compile-time validation
- No runtime type errors
- Serde for serialization

### Testing Strategy
- **Unit tests**: In each module/crate
- **Integration tests**: Cross-crate workflows
- **E2E tests**: Full API with Docker containers

### Database Features
- Embedded SurrealDB with RocksDB backend
- Schema enforcement with validation
- Custom indexes for performance
- Migration system for versioning

## Performance

- Parallel test execution with nextest
- Connection pooling via SurrealDB
- Async/await throughout the stack
- Zero-copy deserialization where possible

## License

MIT

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass: `cargo nextest run`
5. Submit a pull request
