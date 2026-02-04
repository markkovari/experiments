# Axum + PostgreSQL + JWT Authentication Demo

A comprehensive Rust web application demonstrating different architectural patterns using Axum, PostgreSQL, JWT authentication, and comprehensive testing with Testcontainers.

## 📋 Project Overview

This project demonstrates:
- **Two architectural approaches**: OOP (Users module) and Functional (Posts module)
- **JWT Authentication**: Token-based auth with Argon2 password hashing
- **Multiple testing levels**: Unit, Integration, API, and E2E tests
- **Modern Rust tooling**: Axum, SQLx, Tokio, Testcontainers
- **Real database testing**: PostgreSQL via Testcontainers

## 🏗️ Architecture

### Users Module (OOP Approach)
Located in `src/modules/users/`:
- **Domain**: User struct with business logic
- **Repository**: Trait-based repository pattern with SQLx
- **Service**: Business operations and JWT handling
- **Handlers**: Axum HTTP handlers
- **Routes**: Axum router configuration

### Posts Module (Functional Approach)
Located in `src/modules/posts/`:
- **Domain**: Type definitions and pure functions
- **Repository**: Factory functions returning repository
- **Use Cases**: Pure functions for business operations
- **Handlers**: Functional HTTP handlers
- **Routes**: Functional route composition

### Authentication
- **JWT tokens** with configurable expiration
- **Argon2 password hashing** for security
- **Middleware extractors**: `RequireAuth` and `OptionalAuth`
- **Login/Register endpoints** with validation

## 🧪 Testing Levels

### 1. Unit Tests (`tests/unit/`)
**Purpose**: Test individual units of code in isolation with mocked dependencies.

**What's tested:**
- Business logic without external dependencies
- Domain models and their methods
- Pure functions and transformations
- Password hashing and JWT token generation

**No database, no HTTP requests**

**Run:**
```bash
cargo test --lib
```

### 2. Integration Tests (`tests/integration/`)
**Purpose**: Test repository layer with real PostgreSQL database via Testcontainers.

**What's tested:**
- Database operations (CRUD)
- SQLx queries and transactions
- Data persistence and retrieval
- Database constraints and relationships

**Real database via Testcontainers, no HTTP layer**

**Run:**
```bash
cargo test --test integration
```

### 3. API Tests (`tests/api/`)
**Purpose**: Test HTTP endpoints end-to-end with real database.

**What's tested:**
- HTTP routes and methods
- Request/response handling
- Authentication middleware
- Status codes and error responses
- Full request/response cycle

**Real database + Real HTTP (full stack)**

**Run:**
```bash
cargo test --test api
```

### 4. E2E Tests (`tests/e2e/`)
**Purpose**: Test complete user workflows through the entire application stack.

**What's tested:**
- Complete user journeys (register → login → CRUD operations)
- Multiple operations in sequence
- Cross-module interactions
- Real-world scenarios with authentication

**Real database + Real HTTP (full stack)**

**Run:**
```bash
cargo test --test e2e
```

## 🚀 Getting Started

### Prerequisites
- Rust 1.75+ (2021 edition)
- PostgreSQL 14+ (or use Testcontainers for development)
- Docker (for Testcontainers - required for tests)

### Installation

```bash
# Clone the repository
cd axum-test-it-is-whay-it-is

# Install dependencies
cargo build

# Set up environment variables
cp .env.example .env
# Edit .env with your configuration

# Run migrations (requires PostgreSQL running)
sqlx migrate run
```

### Running the Application

```bash
# Development mode
cargo run

# Build for production
cargo build --release

# Run production build
./target/release/axum-test-it-is-whay-it-is
```

The server will start on `http://localhost:3000` (configurable via `SERVER_ADDR` in `.env`).

## 🧪 Running Tests

This project uses **cargo-nextest** for faster, more reliable test execution.

```bash
# Install nextest (if not already installed)
cargo install cargo-nextest

# Run all tests
cargo nextest run

# Run with specific profile
cargo nextest run --profile ci

# Run specific test types
cargo nextest run --lib              # Unit tests only
cargo nextest run --test auth_tests  # Specific test file

# Run with output
cargo nextest run --success-output immediate

# Run specific test by name
cargo nextest run test_name

# Traditional cargo test (still works)
cargo test
```

**Nextest Benefits:**
- ⚡ Faster execution (parallel by default)
- 📊 Better output formatting
- 🔄 Automatic retry for flaky tests (in CI profile)
- 🎯 Per-test timing information

**Note**: Integration, API, and E2E tests require Docker to be running (for Testcontainers).

## 📁 Project Structure

```
├── src/
│   ├── auth/                   # JWT and password handling
│   │   ├── jwt.rs             # Token generation & verification
│   │   └── password.rs        # Argon2 password hashing
│   ├── middleware/            # Axum middleware
│   │   ├── auth.rs           # Auth extractors (RequireAuth, OptionalAuth)
│   │   └── validation.rs     # Request validation
│   ├── modules/
│   │   ├── users/            # Users module (OOP approach)
│   │   │   ├── domain.rs    # User entity and DTOs
│   │   │   ├── repository.rs # Data access layer
│   │   │   ├── service.rs   # Business logic
│   │   │   ├── handlers.rs  # HTTP handlers
│   │   │   └── routes.rs    # Route definitions
│   │   └── posts/            # Posts module (Functional approach)
│   │       ├── domain.rs    # Types and pure functions
│   │       ├── repository.rs # Repository factory
│   │       ├── use_cases.rs # Business operations
│   │       ├── handlers.rs  # HTTP handler functions
│   │       └── routes.rs    # Route composition
│   ├── shared/
│   │   ├── error.rs         # Error types and handling
│   │   └── response.rs      # Response wrappers
│   ├── lib.rs               # Library root & app setup
│   └── main.rs              # Binary entry point
│
├── migrations/                # SQLx migrations
│   ├── 20250203_create_users_table.sql
│   └── 20250203_create_posts_table.sql
│
├── tests/
│   ├── unit/                 # Unit tests
│   ├── integration/          # Integration tests (real DB)
│   ├── api/                  # API tests (full stack)
│   └── e2e/                  # E2E tests (complete workflows)
│
├── Cargo.toml                # Rust dependencies
├── .env                      # Environment variables
└── README.md                 # This file
```

## 🔑 Key Differences Between Test Types

| Aspect | Unit | Integration | API | E2E |
|--------|------|-------------|-----|-----|
| **Speed** | ⚡ Fastest | 🐢 Slower | 🐢 Slower | 🐢 Slowest |
| **Database** | ❌ Mocked | ✅ Real (Testcontainers) | ✅ Real (Testcontainers) | ✅ Real (Testcontainers) |
| **HTTP Layer** | ❌ No | ❌ No | ✅ Yes (axum::test) | ✅ Yes (axum::test) |
| **Scope** | Single function | Repository + DB | Full endpoint | Full workflow |
| **Dependencies** | All mocked | DB real, others mocked | Everything real | Everything real |
| **Isolation** | ✅ High | ⚠️ Medium | ⚠️ Medium | ❌ Low |
| **Best For** | Logic testing | Data layer testing | API contract testing | User journey testing |

## 🛠️ Technologies Used

- **Runtime**: Tokio (async runtime)
- **Framework**: Axum 0.7
- **Database**: PostgreSQL + SQLx
- **Authentication**: JWT (jsonwebtoken) + Argon2
- **Validation**: validator crate
- **Testing**: Testcontainers + tokio-test
- **Tracing**: tracing + tracing-subscriber

## 📚 API Endpoints

### Authentication
- `POST /api/users/register` - Register new user
- `POST /api/users/login` - Login and get JWT token

### Users (Protected)
- `GET /api/users/profile` - Get current user profile (requires auth)
- `GET /api/users` - Get all users (requires auth)
- `GET /api/users/:id` - Get user by ID (requires auth)
- `PUT /api/users/:id` - Update user (requires auth)
- `DELETE /api/users/:id` - Delete user (requires auth)

### Posts
- `GET /api/posts` - Get all posts (public)
- `GET /api/posts/published` - Get published posts (public)
- `GET /api/posts/:id` - Get post by ID (public)
- `GET /api/posts/author/:authorId` - Get posts by author (public)
- `POST /api/posts` - Create post (requires auth)
- `PUT /api/posts/:id` - Update post (requires auth, only author)
- `DELETE /api/posts/:id` - Delete post (requires auth, only author)

### System
- `GET /health` - Health check

## 🎯 Testing Best Practices Demonstrated

1. **Test Pyramid**: More unit tests, fewer E2E tests
2. **Isolation**: Each test type has clear boundaries
3. **Real Dependencies**: Integration/E2E tests use real PostgreSQL via Testcontainers
4. **Mocking Strategy**: Only mock what you need to
5. **Clean State**: Database cleanup between tests
6. **Descriptive Names**: Clear test descriptions
7. **Arrange-Act-Assert**: Consistent test structure

## 🔒 Security Features

- **Password Hashing**: Argon2 for secure password storage
- **JWT Tokens**: Stateless authentication with expiration
- **Authorization**: Route-level and resource-level authorization
- **Validation**: Input validation using validator crate
- **Error Handling**: Secure error messages (no sensitive data leakage)

## 📝 Environment Variables

```bash
DATABASE_URL=postgres://user:password@localhost:5432/dbname
JWT_SECRET=your_secret_key_at_least_32_characters
SERVER_ADDR=0.0.0.0:3000
RUST_LOG=info,axum_test_it_is_whay_it_is=debug
```

## 🚧 Development

This project includes a **justfile** for convenient command shortcuts.

### Install just (optional but recommended)
```bash
cargo install just
```

### Common Commands
```bash
# Show all available commands
just

# Run tests (uses nextest)
just test              # or: just t

# Build project
just build             # or: just b

# Run server
just run               # or: just r

# Run in watch mode (requires cargo-watch)
just dev

# Run all CI checks (format, clippy, test)
just ci

# Format code
just fmt

# Run clippy
just clippy

# Show project info
just info

# Count lines of code
just loc
```

### Without just (traditional cargo commands)
```bash
# Watch mode for development
cargo watch -x run

# Format code
cargo fmt

# Lint
cargo clippy

# Check without building
cargo check

# Run tests with nextest
cargo nextest run
```

See `TESTING.md` for detailed testing documentation.

## 📄 License

MIT

## 🤝 Contributing

Contributions are welcome! This is a learning/demo project showcasing different patterns in Rust web development.
