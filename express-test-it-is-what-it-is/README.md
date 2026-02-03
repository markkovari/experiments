# Express + PostgreSQL Testing Levels Demo

A comprehensive demonstration of different testing levels for an Express.js application using PostgreSQL, Prisma ORM, and Vitest with Testcontainers.

## 📋 Project Overview

This project demonstrates:
- **Two architectural approaches**: OOP (Users module) and Functional (Posts module)
- **Four testing levels**: Unit, Integration, API, and E2E tests
- **Modern tooling**: TypeScript, Prisma 7, Vitest, Testcontainers
- **Real database testing**: PostgreSQL via Testcontainers

## 🏗️ Architecture

### Users Module (OOP Approach)
Located in `src/modules/users/`:
- **Domain**: `User` class with business logic
- **Repository**: `UserRepository` class (interface + implementation)
- **Service**: `UserService` class for business operations
- **Controller**: `UserController` class for HTTP handling
- **Routes**: Express router configuration

### Posts Module (Functional Approaches)
Located in `src/modules/posts/`:

**Two functional implementations available:**

#### 1. Original: Higher-Order Functions (HOF) with Dependency Injection
- **Domain**: Type definitions and pure functions
- **Repository**: Factory functions returning repository interface
- **Use Cases**: HOF that return use case functions (currying)
- **Handlers**: Factory functions for HTTP handling
- **Routes**: Functional route composition
- **Mounted at**: `/api/posts`

#### 2. Alternative: Direct Functions with Singleton Pattern
- **Domain**: Same types and pure functions
- **Repository**: Same factory functions
- **Use Cases (Alternative)**: Direct functions using module-level repository
- **Handlers (Alternative)**: Direct handler functions
- **Routes (Alternative)**: Direct route configuration
- **Mounted at**: `/api/posts-alt`

Both expose the same API but with different dependency management patterns!
See `docs/FUNCTIONAL_PATTERNS_COMPARISON.md` for detailed comparison.

## 🧪 Testing Levels

### 1. Unit Tests (`tests/unit/`)
**Purpose**: Test individual units of code in isolation with mocked dependencies.

**What's tested:**
- Business logic without external dependencies
- Domain models and their methods
- Pure functions and transformations

**No database, no HTTP requests**

**Examples:**
- `tests/unit/users/user-service.unit.test.ts` - Tests UserService with mocked repository
- `tests/unit/users/user-domain.unit.test.ts` - Tests User domain model
- `tests/unit/posts/create-post-use-case.unit.test.ts` - Tests post creation use case
- `tests/unit/posts/update-post-use-case.unit.test.ts` - Tests post update logic

**Run:**
```bash
pnpm test:unit
```

### 2. Integration Tests (`tests/integration/`)
**Purpose**: Test repository layer with real PostgreSQL database.

**What's tested:**
- Database operations (CRUD)
- Prisma queries and mutations
- Data persistence and retrieval
- Database constraints and relationships

**Real database via Testcontainers, no HTTP layer**

**Examples:**
- `tests/integration/users/user-repository.integration.test.ts` - Tests UserRepository with real DB
- `tests/integration/posts/post-repository.integration.test.ts` - Tests PostRepository with real DB

**Run:**
```bash
pnpm test:integration
```

### 3. API Tests (`tests/api/`)
**Purpose**: Test HTTP endpoints with mocked database layer.

**What's tested:**
- HTTP routes and methods
- Request/response handling
- Status codes and error responses
- Input validation at HTTP level
- Controller logic

**Mocked database, real HTTP via Supertest**

**Examples:**
- `tests/api/users/user-endpoints.api.test.ts` - Tests all user endpoints (GET, POST, PUT, DELETE)
- `tests/api/posts/post-endpoints.api.test.ts` - Tests all post endpoints

**Run:**
```bash
pnpm test:api
```

### 4. E2E Tests (`tests/e2e/`)
**Purpose**: Test complete user workflows through the entire application stack.

**What's tested:**
- Complete user journeys
- Multiple operations in sequence
- Cross-module interactions
- Real-world scenarios

**Real database + Real HTTP (full stack)**

**Examples:**
- `tests/e2e/users/user-workflows.e2e.test.ts` - Tests complete user CRUD workflows
- `tests/e2e/posts/post-workflows.e2e.test.ts` - Tests post publishing workflows

**Run:**
```bash
pnpm test:e2e
```

## 🚀 Getting Started

### Prerequisites
- Node.js 18+
- pnpm
- Docker (for Testcontainers - required for integration and E2E tests)

### Installation

```bash
# Install dependencies
pnpm install

# Generate Prisma client
pnpm prisma:generate

# Set up database (local Prisma Postgres or your own PostgreSQL)
# The DATABASE_URL is already configured in .env
```

### Running the Application

```bash
# Development mode with hot reload
pnpm dev

# Build for production
pnpm build

# Run production build
pnpm start
```

## 🧪 Running Tests

```bash
# Run all tests
pnpm test

# Run specific test levels
pnpm test:unit          # Unit tests only
pnpm test:integration   # Integration tests only
pnpm test:api           # API tests only
pnpm test:e2e           # E2E tests only

# Run tests with UI
pnpm test:ui
```

## 📁 Project Structure

```
├── src/
│   ├── modules/
│   │   ├── users/              # OOP approach
│   │   │   ├── domain/         # User entity
│   │   │   ├── repository/     # Data access layer
│   │   │   ├── services/       # Business logic
│   │   │   ├── controllers/    # HTTP handlers
│   │   │   └── routes/         # Route definitions
│   │   └── posts/              # Functional approach
│   │       ├── domain/         # Types and pure functions
│   │       ├── repository/     # Repository factory
│   │       ├── use-cases/      # Business operations
│   │       ├── handlers/       # HTTP handler factories
│   │       └── routes/         # Route composition
│   ├── shared/
│   │   └── prisma/             # Prisma client singleton
│   ├── app.ts                  # Express app setup
│   └── server.ts               # Server entry point
│
├── tests/
│   ├── unit/                   # Unit tests (mocked dependencies)
│   ├── integration/            # Integration tests (real DB)
│   ├── api/                    # API tests (mocked DB, real HTTP)
│   ├── e2e/                    # E2E tests (full stack)
│   └── setup/                  # Test utilities
│       ├── testcontainers.ts   # DB container setup
│       └── global-setup.ts     # Vitest global setup
│
├── prisma/
│   └── schema.prisma           # Database schema
├── prisma.config.ts            # Prisma 7 configuration
├── vitest.config.ts            # Vitest configuration
└── tsconfig.json               # TypeScript configuration
```

## 🔑 Key Differences Between Test Types

| Aspect | Unit | Integration | API | E2E |
|--------|------|-------------|-----|-----|
| **Speed** | ⚡ Fastest | 🐢 Slower | ⚡ Fast | 🐢 Slowest |
| **Database** | ❌ Mocked | ✅ Real (Testcontainers) | ❌ Mocked | ✅ Real (Testcontainers) |
| **HTTP Layer** | ❌ No | ❌ No | ✅ Yes (Supertest) | ✅ Yes (Supertest) |
| **Scope** | Single function/class | Repository + DB | Controller + Routes | Full workflow |
| **Dependencies** | All mocked | DB real, others mocked | DB mocked, HTTP real | Everything real |
| **Isolation** | ✅ High | ⚠️ Medium | ⚠️ Medium | ❌ Low |
| **Best For** | Logic testing | Data layer testing | API contract testing | User journey testing |

## 🛠️ Technologies Used

- **Runtime**: Node.js + TypeScript
- **Framework**: Express.js 5
- **ORM**: Prisma 7
- **Database**: PostgreSQL
- **Testing**: Vitest + Supertest + Testcontainers
- **Package Manager**: pnpm
- **Test Utilities**: Custom Test App Factory for focused testing

## 📚 API Endpoints

### Users (OOP)
- `GET /api/users` - Get all users
- `GET /api/users/:id` - Get user by ID
- `POST /api/users` - Create user
- `PUT /api/users/:id` - Update user
- `DELETE /api/users/:id` - Delete user

### Posts (Functional - Two Implementations)

**Original (HOF)** at `/api/posts`:
- `GET /api/posts` - Get all posts
- `GET /api/posts/published` - Get published posts
- `GET /api/posts/:id` - Get post by ID
- `GET /api/posts/author/:authorId` - Get posts by author
- `POST /api/posts` - Create post
- `PUT /api/posts/:id` - Update post
- `DELETE /api/posts/:id` - Delete post

**Alternative (Direct)** at `/api/posts-alt`:
- Same endpoints, different implementation
- Uses direct functions with singleton pattern

## 🎯 Testing Best Practices Demonstrated

1. **Test Pyramid**: More unit tests, fewer E2E tests
2. **Isolation**: Each test type has clear boundaries
3. **Real Dependencies**: Integration/E2E tests use real PostgreSQL via Testcontainers
4. **Mocking Strategy**: Only mock what you need to
5. **Clean State**: Database cleanup between tests
6. **Descriptive Names**: Clear test descriptions
7. **Arrange-Act-Assert**: Consistent test structure
8. **Focused Testing**: Test App Factory for minimal, isolated API tests

## 🔬 Test App Factory

This project includes a custom **Test App Factory** for creating minimal Express apps in tests:

```typescript
import { createSingleRouteApp } from '../../helpers/test-app-factory';

// Load ONLY the routes you're testing
const app = createSingleRouteApp('/api/users', createUserRoutes(controller));
```

**Benefits:**
- ⚡ **10-20x faster** than loading full app
- 🎯 **Better isolation** - only loads what you test
- 📝 **Clearer intent** - explicit about dependencies
- 🔧 **More control** - compose exactly what you need

See `docs/TEST_APP_FACTORY_GUIDE.md` for complete guide with examples.

## 📝 Notes

- **Prisma 7**:
  - Uses `prisma.config.ts` for database URL configuration (not in schema)
  - Uses adapter pattern (`@prisma/adapter-pg`) for dynamic database connections in tests
  - No `url` property in `schema.prisma`
- **Testcontainers**: Automatically manages PostgreSQL containers for integration/E2E tests
- **Vitest**: Fast test runner with great TypeScript support
- **Type Safety**: Full TypeScript coverage across the application

## ✅ Test Results

All test levels are working:

```bash
# Unit Tests: ✅ 25 passing (5 files)
# API Tests: ✅ 23 passing (2 files)
# Integration Tests: ✅ 10+ passing (2 files)
# E2E Tests: ✅ 5+ passing (2 files)
```
