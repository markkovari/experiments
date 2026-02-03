# Testing Guide

## 🎯 Quick Start

```bash
# Run all unit tests (fastest, no dependencies)
pnpm test:unit

# Run API tests (HTTP layer, mocked database)
pnpm test:api

# Run integration tests (real database via testcontainers)
pnpm test:integration

# Run E2E tests (full stack)
pnpm test:e2e

# Run all tests
pnpm test
```

## 📊 Test Type Comparison

### 1. Unit Tests (`tests/unit/`)

**Speed:** ⚡⚡⚡ Very Fast (< 1 second)
**Purpose:** Test business logic in isolation
**Dependencies:** None (all mocked)

**What they test:**
- Service layer logic (UserService methods)
- Domain model behavior (User class methods)
- Use case functions (createPost, updatePost)
- Pure functions (toPostDTO)

**Example:**
```typescript
// Test UserService.createUser with mocked repository
it('should create a user when email is unique', async () => {
  vi.mocked(mockRepository.findByEmail).mockResolvedValue(null);
  vi.mocked(mockRepository.create).mockResolvedValue(createdUser);

  const result = await userService.createUser(userData);

  expect(result).toEqual(createdUser);
});
```

**When to use:**
- Testing business rules
- Validating edge cases
- Fast feedback during development

---

### 2. API Tests (`tests/api/`)

**Speed:** ⚡⚡ Fast (< 1 second)
**Purpose:** Test HTTP endpoints and controllers
**Dependencies:** Mocked database, real HTTP (via Supertest)

**What they test:**
- HTTP routes (GET, POST, PUT, DELETE)
- Status codes (200, 404, 409, etc.)
- Request/response format
- Input validation
- Error handling

**Example:**
```typescript
// Test POST /api/users endpoint
it('should create a new user', async () => {
  vi.mocked(mockUserService.createUser).mockResolvedValue(newUser);

  const response = await request(app)
    .post('/api/users')
    .send({ email: 'new@example.com', name: 'New User', password: 'pass' });

  expect(response.status).toBe(201);
  expect(response.body.email).toBe('new@example.com');
});
```

**When to use:**
- Verifying API contracts
- Testing HTTP layer logic
- Validating request/response handling

---

### 3. Integration Tests (`tests/integration/`)

**Speed:** 🐢 Slower (~10 seconds)
**Purpose:** Test integration between layers with real PostgreSQL
**Dependencies:** Real PostgreSQL (via Testcontainers), no HTTP

**What they test:**

#### A. Repository Integration (`*-repository.integration.test.ts`)
- CRUD operations
- Database queries
- Data persistence
- Schema validation

#### B. Service/Use Case Integration (`*-service.integration.test.ts`, `*-use-cases.integration.test.ts`)
- **Business logic + Database** together
- Validation + Database constraints
- Cascade operations
- Transaction-like scenarios
- Foreign key constraints

**Key difference from Unit tests:**
- Unit: `Service → MOCKED Repository`
- Integration: `Service → REAL Repository → REAL Database`

**Example (Repository):**
```typescript
// Test UserRepository.create with real database
it('should create a user in the database', async () => {
  const user = await userRepository.create({
    email: 'test@example.com',
    name: 'Test User',
    password: 'password123'
  });

  expect(user.id).toBeDefined();
  expect(user.email).toBe('test@example.com');
});
```

**Example (Service Integration):**
```typescript
// Test UserService + Repository + Database together
it('should enforce email uniqueness at business logic AND database level', async () => {
  await userService.createUser({
    email: 'test@example.com',
    name: 'User 1',
    password: 'pass'
  });

  // Business logic check should catch this
  await expect(
    userService.createUser({
      email: 'test@example.com', // duplicate
      name: 'User 2',
      password: 'pass'
    })
  ).rejects.toThrow('User with this email already exists');

  // Verify only one user in database
  const allUsers = await userService.getAllUsers();
  expect(allUsers).toHaveLength(1);
});
```

**When to use:**
- Verifying database schema
- Testing complex queries
- Validating business logic + database interaction
- Testing cascade operations
- Verifying constraint enforcement

---

### 4. E2E Tests (`tests/e2e/`)

**Speed:** 🐢🐢 Slowest (~10 seconds)
**Purpose:** Test complete user workflows
**Dependencies:** Real PostgreSQL + Real HTTP (full stack)

**What they test:**
- Complete user journeys
- Multi-step workflows
- Cross-module interactions
- Data consistency across operations

**Example:**
```typescript
// Test complete user CRUD workflow
it('should create, read, update, and delete a user', async () => {
  // Create
  const createRes = await request(app).post('/api/users').send(userData);
  const userId = createRes.body.id;

  // Read
  const getRes = await request(app).get(`/api/users/${userId}`);
  expect(getRes.body.email).toBe('test@example.com');

  // Update
  await request(app).put(`/api/users/${userId}`).send({ name: 'Updated' });

  // Delete
  await request(app).delete(`/api/users/${userId}`);
  const deletedRes = await request(app).get(`/api/users/${userId}`);
  expect(deletedRes.status).toBe(404);
});
```

**When to use:**
- Validating user workflows
- Testing feature completeness
- Pre-deployment smoke tests

---

## 🔧 Technical Details

### Testcontainers Setup

Integration and E2E tests use Testcontainers to spin up real PostgreSQL:

```typescript
// tests/setup/testcontainers.ts
import { PostgreSqlContainer } from '@testcontainers/postgresql';
import { PrismaPg } from '@prisma/adapter-pg';
import { Pool } from 'pg';

const container = await new PostgreSqlContainer('postgres:16-alpine')
  .withDatabase('testdb')
  .start();

const pool = new Pool({ connectionString: container.getConnectionUri() });
const adapter = new PrismaPg(pool);
const prisma = new PrismaClient({ adapter });
```

### Prisma 7 Adapter Pattern

Prisma 7 uses adapters for dynamic database connections:

```typescript
// For tests: Use adapter with connection string
import { PrismaPg } from '@prisma/adapter-pg';
const prisma = new PrismaClient({ adapter: new PrismaPg(pool) });

// For production: Uses DATABASE_URL from prisma.config.ts
const prisma = new PrismaClient();
```

---

## 📈 Test Pyramid Strategy

```
       /\
      /  \      E2E Tests (5+ tests)
     /____\     Few, slow, high confidence
    /      \
   /  API   \   API Tests (23 tests)
  /__________\  More, faster, focused
 /            \
/ Integration  \ Integration Tests (10+ tests)
/________________\ Database focused
/                \
/   Unit Tests    \ Unit Tests (25 tests)
/                  \ Many, fast, isolated
```

**Recommended distribution:**
- 50% Unit Tests (fast, many)
- 20% Integration Tests (database layer)
- 20% API Tests (HTTP layer)
- 10% E2E Tests (critical workflows)

---

## 🎓 Best Practices

1. **Run unit tests frequently** during development
2. **Run API tests** before committing
3. **Run integration tests** before pushing
4. **Run E2E tests** in CI/CD pipeline
5. **Use mocks strategically** - only mock what you need to
6. **Clean state between tests** - use beforeEach hooks
7. **Test one thing at a time** - keep tests focused
8. **Write descriptive test names** - describe the behavior

---

## 🐛 Troubleshooting

### Docker not running
```bash
# Start Docker Desktop
# Or ensure Docker daemon is running
docker ps
```

### Testcontainers timeout
```bash
# Increase timeout in vitest.config.ts
testTimeout: 60000  // 60 seconds
```

### Port conflicts
```bash
# Testcontainers automatically assigns random ports
# No configuration needed
```

### Database migrations
```bash
# Migrations are in prisma/migrations/
# Applied automatically during test setup via:
# execSync('pnpm prisma migrate deploy')
```
