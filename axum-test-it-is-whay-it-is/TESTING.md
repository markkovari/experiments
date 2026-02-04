# Testing Guide

This document explains the testing setup and best practices for this project.

## Quick Start

```bash
# Install nextest (faster test runner)
cargo install cargo-nextest

# Run all tests
cargo nextest run

# Or using justfile
just test
```

## Test Runner: Nextest

This project uses **cargo-nextest** instead of the standard `cargo test` for better performance and output.

### Why Nextest?

- ⚡ **2-3x faster** - Tests run in parallel more efficiently
- 📊 **Better output** - Cleaner, more informative test results
- 🔄 **Retry support** - Automatic retry for flaky tests (CI profile)
- 🎯 **Per-test timing** - See which tests are slow
- 🔍 **Better filtering** - More flexible test selection

### Nextest Configuration

Configuration is in `.config/nextest.toml`:

```toml
[profile.default]
test-threads = "num-cpus"
failure-output = "immediate"
slow-timeout = { period = "60s" }

[profile.ci]
retries = 2
fail-fast = false
```

### Nextest Commands

```bash
# Run all tests
cargo nextest run

# Run with specific profile
cargo nextest run --profile ci

# Run only unit tests
cargo nextest run --lib

# Run specific test file
cargo nextest run --test auth_tests

# Run with verbose output
cargo nextest run --success-output immediate

# Run specific test by name
cargo nextest run test_password_hashing

# Run tests in watch mode (requires cargo-watch)
cargo watch -x "nextest run"
```

## Test Organization

### Current Tests (7 passing)

**Unit Tests** (`src/`)
- `src/auth/jwt.rs` - JWT token tests (3 tests)
- `src/auth/password.rs` - Password hashing tests (1 test)

**Integration Tests** (`tests/`)
- `tests/auth_tests.rs` - Auth integration tests (3 tests)

### Test Structure

```
tests/
├── auth_tests.rs           # Auth-related tests
├── README.md               # Test documentation
└── (future tests)
```

## Writing Tests

### Unit Tests

Unit tests are embedded in the source files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let result = my_function();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Integration tests are in separate files in `tests/`:

```rust
use axum_test_it_is_whay_it_is::auth::jwt;

#[tokio::test]
async fn test_jwt_workflow() {
    let token = jwt::generate_token(...).unwrap();
    let claims = jwt::verify_token(&token, secret).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
}
```

### Test Helpers

Test helpers are available in `src/lib.rs`:

```rust
#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers {
    pub async fn create_test_app(pool: PgPool) -> Router {
        // Creates a test instance of the app
    }
}
```

## Running Tests

### With Nextest (Recommended)

```bash
# All tests
just test
# or
cargo nextest run

# Unit tests only
just test-unit
# or
cargo nextest run --lib

# With output
just test-verbose
# or
cargo nextest run --success-output immediate

# CI profile (with retries)
just test-ci
# or
cargo nextest run --profile ci
```

### With Standard Cargo Test

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

### Watch Mode

```bash
# Using just
just test-watch

# Or directly
cargo watch -x "nextest run"
```

## Test Profiles

### Default Profile
- Uses all available CPUs
- Immediate failure output
- 60-second timeout for slow tests

### CI Profile
- Retries flaky tests 2 times
- Doesn't fail fast (runs all tests)
- More detailed output

### Integration Profile (for future use)
- Limited parallelism (4 threads)
- Longer timeout (120 seconds)
- For tests with external dependencies

## Test Coverage

Current coverage (7 tests):
- ✅ JWT generation and verification
- ✅ JWT error handling (invalid tokens, wrong secrets)
- ✅ Password hashing with Argon2
- ✅ Password verification

To add coverage tracking:
```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html

# Open report
open target/llvm-cov/html/index.html
```

## CI/CD Integration

The project includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that:
- Runs on every push and PR
- Uses nextest for faster execution
- Runs format checks, clippy, and tests
- Generates code coverage reports

```yaml
- name: Run tests
  run: cargo nextest run --profile ci
```

## Test Best Practices

### 1. Test Naming
- Use descriptive names: `test_jwt_generation_and_verification`
- Prefix integration tests: `test_create_user_integration`

### 2. Test Structure (AAA Pattern)
```rust
#[test]
fn test_something() {
    // Arrange - Set up test data
    let input = "test";

    // Act - Execute the code
    let result = function(input);

    // Assert - Verify the outcome
    assert_eq!(result, expected);
}
```

### 3. Async Tests
Use `#[tokio::test]` for async functions:
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### 4. Test Isolation
- Each test should be independent
- Clean up resources after tests
- Use unique test data to avoid conflicts

### 5. Error Testing
```rust
#[test]
fn test_error_case() {
    let result = function_that_fails();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Expected error");
}
```

## Future Test Additions

### Integration Tests with Testcontainers
```rust
use testcontainers::*;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
async fn test_user_repository() {
    let container = Postgres::default().start().await;
    // ... test with real database
}
```

### API Tests
```rust
use axum::body::Body;
use axum::http::Request;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_register_endpoint() {
    let app = create_test_app(pool).await;
    let request = Request::post("/api/users/register")
        .body(Body::from(json_body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

### E2E Tests
```rust
#[tokio::test]
async fn test_complete_user_workflow() {
    // 1. Register
    // 2. Login
    // 3. Make authenticated request
    // 4. Verify response
}
```

## Troubleshooting

### Tests Timeout
- Increase timeout in `.config/nextest.toml`
- Check for deadlocks or infinite loops
- Use `--success-output immediate` to see which test hangs

### Flaky Tests
- Use CI profile with retries: `just test-ci`
- Add proper synchronization
- Avoid time-dependent assertions

### Docker Not Running
- Tests requiring Docker will fail
- Start Docker: `docker ps`
- Or skip those tests: `cargo nextest run --lib`

## Resources

- [Nextest Documentation](https://nexte.st/)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
