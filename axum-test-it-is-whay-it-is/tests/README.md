# Test Structure

## Current Tests (32 passing ✅)

### Unit Tests (13 tests)
- **Auth tests** in `src/auth/*/tests/` modules and `tests/unit/auth_tests.rs`
  - Password hashing and verification (2 tests in lib, 1 test in unit tests)
  - JWT token generation and verification (3 tests in lib, 3 tests in unit tests)

- **Service mock tests** in `tests/unit/user_service_mock_tests.rs`
  - User service with mocked repository (6 tests)
  - Uses mockall for true unit testing without database
  - Tests create, get, login, and error scenarios

These tests run without any external dependencies and are very fast (~1.5s).

### Integration Tests (15 tests)
- **User Repository** (`tests/integration/user_repository_tests.rs`) - 7 tests
  - CRUD operations with real PostgreSQL
  - Email uniqueness constraints
  - Cascade relationships

- **Post Repository** (`tests/integration/post_repository_tests.rs`) - 8 tests
  - CRUD operations with real PostgreSQL
  - Published/unpublished filtering
  - Author relationships
  - Cascade delete verification

These tests use Testcontainers to spin up PostgreSQL instances (~5-6s per test).

### E2E Tests (4 tests)
- **User Workflows** (`tests/e2e/user_workflow_tests.rs`) - 4 tests
  - Complete registration → login → authenticated request flow
  - Unauthorized access handling
  - Invalid credentials handling
  - Health check endpoint

These tests run against the full application stack with real database (~2-3s per test).

## Test Organization

Tests are organized into three categories using Rust's integration test structure:

- **`tests/unit/`** - Unit tests that mock dependencies (mockall)
- **`tests/integration/`** - Integration tests with real database (testcontainers)
- **`tests/e2e/`** - End-to-end tests with full application stack
- **`tests/common/`** - Shared test utilities (setup_test_db)

Each category has a corresponding `.rs` file in the `tests/` root directory that declares the modules.

## Running Tests

```bash
# Run all tests (recommended - uses nextest)
cargo nextest run --features test-helpers

# Run specific test category
cargo nextest run --features test-helpers unit
cargo nextest run --features test-helpers integration
cargo nextest run --features test-helpers e2e

# Run specific test
cargo nextest run --features test-helpers test_name

# Using standard cargo test
cargo test --features test-helpers

# Run with output
cargo test --features test-helpers -- --nocapture
```

## Adding New Tests

### Unit Tests (with Mocks)

Create tests in `tests/unit/` directory and use mockall to mock dependencies:

```rust
use mockall::mock;
use axum_test_it_is_whay_it_is::modules::users::{
    repository::UserRepositoryTrait,
    service::UserService,
};

mock! {
    pub UserRepository {}
    #[async_trait::async_trait]
    impl UserRepositoryTrait for UserRepository {
        // Define trait methods here
    }
    impl Clone for UserRepository {
        fn clone(&self) -> Self;
    }
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock_repo = MockUserRepository::new();
    mock_repo.expect_create().returning(|_| Ok(user));
    let service = UserService::new(mock_repo, "secret".to_string());
    // Test service methods
}
```

### Integration Tests (with Real Database)

Create tests in `tests/integration/` directory:

```rust
#[tokio::test]
async fn test_with_database() {
    let (_container, pool) = super::common::setup_test_db().await;
    let repo = UserRepository::new(pool.clone());
    // Test repository methods
    pool.close().await;
}
```

### E2E Tests (with Full App)

Create tests in `tests/e2e/` directory:

```rust
use axum_test_it_is_whay_it_is::test_helpers::create_test_app;

#[tokio::test]
async fn test_api_endpoint() {
    let (_container, pool) = super::common::setup_test_db().await;
    let app = create_test_app(pool.clone()).await;
    // Make HTTP requests using tower::ServiceExt
    pool.close().await;
}
```

## Test Database Setup

Integration and E2E tests use Testcontainers to automatically spin up PostgreSQL instances:

- `tests/common/mod.rs` provides `setup_test_db()` function
- Each test gets a fresh PostgreSQL container
- Migrations are automatically run
- Containers are cleaned up after tests

## Current Test Coverage

✅ **Unit Tests (13 tests)**
- JWT generation and verification (6 tests)
- Password hashing with Argon2 (3 tests)
- Service layer with mocked repositories (6 tests)

✅ **Integration Tests (15 tests)**
- User repository CRUD operations (7 tests)
- Post repository operations with relationships (8 tests)

✅ **E2E Tests (4 tests)**
- Complete authentication workflows (4 tests)

## Notes

- Use `cargo nextest run` for faster parallel test execution
- The `--features test-helpers` flag enables test-only code
- Tests are isolated and run in parallel
- Testcontainers automatically manage database lifecycle
- Mock tests (unit) are fastest, integration tests are slower due to database setup
