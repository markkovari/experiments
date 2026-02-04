# Quick Start Guide

## Prerequisites

1. **Rust** 1.75+ installed
2. **PostgreSQL** 14+ running (or Docker for Testcontainers)
3. **Docker** (for running tests with Testcontainers)

## Setup

### 1. Install Dependencies
```bash
cargo build
```

### 2. Set Up Environment Variables
```bash
cp .env.example .env
# Edit .env with your database credentials
```

### 3. Set Up Database

**Option A: Using existing PostgreSQL**
```bash
# Create database
createdb axum_test

# Update DATABASE_URL in .env
DATABASE_URL=postgres://your_user:your_password@localhost:5432/axum_test

# Run migrations (migrations will run automatically on startup, but you can run them manually)
cargo install sqlx-cli
sqlx migrate run
```

**Option B: Using Docker**
```bash
docker run --name axum-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=axum_test -p 5432:5432 -d postgres:16-alpine
```

### 4. Run the Application
```bash
cargo run
```

The server will start on `http://localhost:3000`

## API Examples

### 1. Register a User
```bash
curl -X POST http://localhost:3000/api/users/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "name": "Test User",
    "password": "password123"
  }'
```

### 2. Login
```bash
curl -X POST http://localhost:3000/api/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'
```

This will return a JWT token. Save it for the next requests.

### 3. Get Profile (Protected Route)
```bash
TOKEN="your_jwt_token_from_login"

curl -X GET http://localhost:3000/api/users/profile \
  -H "Authorization: Bearer $TOKEN"
```

### 4. Create a Post (Protected Route)
```bash
curl -X POST http://localhost:3000/api/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "My First Post",
    "content": "This is the content of my post",
    "published": true
  }'
```

### 5. Get All Posts (Public Route)
```bash
curl -X GET http://localhost:3000/api/posts
```

### 6. Get Published Posts Only (Public Route)
```bash
curl -X GET http://localhost:3000/api/posts/published
```

## Running Tests

This project uses **cargo-nextest** for faster test execution.

### Install Nextest (if not already installed)
```bash
cargo install cargo-nextest
```

### All Tests
```bash
cargo nextest run
```

### Unit Tests Only
```bash
cargo nextest run --lib
```

### Specific Test File
```bash
cargo nextest run --test auth_tests
```

### With Detailed Output
```bash
cargo nextest run --success-output immediate
```

### CI Profile (with retries)
```bash
cargo nextest run --profile ci
```

### Traditional cargo test (still works)
```bash
cargo test
```

**Nextest Benefits:**
- ⚡ 2-3x faster test execution
- 📊 Better progress reporting
- 🔄 Automatic retry for flaky tests (CI profile)
- 🎯 Per-test timing and output

## Project Structure Overview

```
src/
├── auth/               # JWT & password handling
├── middleware/         # Auth extractors & validation
├── modules/
│   ├── users/         # Users module (OOP pattern)
│   └── posts/         # Posts module (Functional pattern)
├── shared/            # Error handling & responses
├── lib.rs            # App setup
└── main.rs           # Entry point
```

## Key Endpoints

### Authentication
- `POST /api/users/register` - Register new user
- `POST /api/users/login` - Login and get JWT

### Users (Protected)
- `GET /api/users/profile` - Get current user profile
- `GET /api/users` - Get all users
- `GET /api/users/:id` - Get user by ID
- `PUT /api/users/:id` - Update user
- `DELETE /api/users/:id` - Delete user

### Posts
- `GET /api/posts` - Get all posts (public)
- `GET /api/posts/published` - Get published posts (public)
- `GET /api/posts/:id` - Get post by ID (public)
- `GET /api/posts/author/:authorId` - Get posts by author (public)
- `POST /api/posts` - Create post (protected)
- `PUT /api/posts/:id` - Update post (protected, author only)
- `DELETE /api/posts/:id` - Delete post (protected, author only)

### System
- `GET /health` - Health check

## Common Issues

### Database Connection Failed
- Make sure PostgreSQL is running
- Check DATABASE_URL in .env
- Verify database exists: `psql -l`

### Migration Errors
- Migrations run automatically on startup
- If you see migration errors, check your database permissions
- Reset database: `dropdb axum_test && createdb axum_test`

### Port Already in Use
- Change SERVER_ADDR in .env
- Kill process on port 3000: `lsof -ti:3000 | xargs kill`

### Test Failures
- Make sure Docker is running (for Testcontainers)
- Tests use isolated databases, so they won't affect your dev database

## Development Workflow

```bash
# Watch mode (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Format code
cargo fmt

# Lint
cargo clippy

# Check without building
cargo check

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Architecture Patterns

This project demonstrates two architectural patterns:

**OOP Pattern (Users Module)**
- Repository trait with implementation
- Service layer for business logic
- Handler functions using state

**Functional Pattern (Posts Module)**
- Repository as factory functions
- Pure use case functions
- Stateless handlers

Both patterns achieve the same goals but show different approaches in Rust.

## Security Notes

- Change JWT_SECRET in production
- Passwords are hashed with Argon2
- JWT tokens expire after 24 hours (configurable in `src/auth/jwt.rs`)
- Use HTTPS in production
- Add rate limiting for production
- Implement refresh tokens for better security

## Next Steps

1. Add more endpoints (e.g., password reset, email verification)
2. Implement pagination for list endpoints
3. Add request logging and metrics
4. Set up CI/CD pipeline
5. Add more comprehensive tests
6. Implement rate limiting
7. Add API documentation (Swagger/OpenAPI)
8. Set up monitoring and alerting

## Resources

- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Tokio Documentation](https://tokio.rs/)
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725)
