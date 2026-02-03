# Test Coverage Comparison

## What Each Test Type Covers

```
┌─────────────────────────────────────────────────────────────┐
│                     Full Application Stack                   │
├─────────────────────────────────────────────────────────────┤
│  HTTP Layer (Routes/Controllers/Handlers)                   │
│  Business Logic (Services/Use Cases)                        │
│  Data Access (Repositories)                                 │
│  Database (PostgreSQL)                                      │
└─────────────────────────────────────────────────────────────┘
```

### Unit Tests
```
┌──────────────────────┐
│  Business Logic      │  ✅ TESTED with mocks
├──────────────────────┤
│  Data Access (MOCK)  │  ❌ Mocked
├──────────────────────┤
│  Database            │  ❌ Not involved
└──────────────────────┘
```
**Tests:** Logic in isolation
**Example:** Does `UserService.createUser()` check for duplicate emails?

---

### Integration Tests - Repository Level
```
┌──────────────────────┐
│  Business Logic      │  ❌ Not involved
├──────────────────────┤
│  Data Access         │  ✅ TESTED
├──────────────────────┤
│  Database            │  ✅ Real PostgreSQL
└──────────────────────┘
```
**Tests:** Data layer + Database
**Example:** Does `UserRepository.create()` correctly insert into database?

---

### Integration Tests - Service/Use Case Level
```
┌──────────────────────┐
│  Business Logic      │  ✅ TESTED
├──────────────────────┤
│  Data Access         │  ✅ Real repository
├──────────────────────┤
│  Database            │  ✅ Real PostgreSQL
└──────────────────────┘
```
**Tests:** Business logic + Data layer + Database
**Example:** Does `UserService.createUser()` enforce uniqueness AND save to database correctly?

---

### API Tests
```
┌──────────────────────┐
│  HTTP Layer          │  ✅ TESTED
├──────────────────────┤
│  Business Logic      │  ✅ Real but thin
├──────────────────────┤
│  Data Access (MOCK)  │  ❌ Mocked
├──────────────────────┤
│  Database            │  ❌ Not involved
└──────────────────────┘
```
**Tests:** HTTP endpoints + Controllers
**Example:** Does `POST /api/users` return 409 on duplicate email?

---

### E2E Tests
```
┌──────────────────────┐
│  HTTP Layer          │  ✅ TESTED
├──────────────────────┤
│  Business Logic      │  ✅ TESTED
├──────────────────────┤
│  Data Access         │  ✅ TESTED
├──────────────────────┤
│  Database            │  ✅ Real PostgreSQL
└──────────────────────┘
```
**Tests:** Complete user workflows
**Example:** Can a user be created, updated, and deleted through the API?

---

## The Key Question: Unit vs Integration?

### "Should I test UserService in Unit or Integration?"

**Answer: BOTH!**

### Unit Test (Fast, Isolated)
```typescript
// UserService → MOCKED Repository
describe('UserService - Unit', () => {
  it('checks business logic', async () => {
    mockRepository.findByEmail.mockResolvedValue(existingUser);

    await expect(
      userService.createUser({ email: 'exists@test.com', ... })
    ).rejects.toThrow('User with this email already exists');

    // Tests: Business logic validates BEFORE calling repository
    expect(mockRepository.create).not.toHaveBeenCalled();
  });
});
```
**Tests:** Business rules work correctly
**Doesn't test:** Does repository actually enforce this? Does database?

---

### Integration Test (Slower, Real DB)
```typescript
// UserService → REAL Repository → REAL Database
describe('UserService - Integration', () => {
  it('enforces uniqueness with real database', async () => {
    await userService.createUser({ email: 'test@test.com', ... });

    await expect(
      userService.createUser({ email: 'test@test.com', ... })
    ).rejects.toThrow('User with this email already exists');

    // Tests: Business logic + Database constraint work together
    const users = await userService.getAllUsers();
    expect(users).toHaveLength(1);
  });
});
```
**Tests:** Business logic + Database constraints work together
**Catches:**
- Database constraint violations
- Foreign key issues
- Transaction problems
- Cascade behavior

---

## Real-World Example: Why Both Are Needed

### Scenario: Email Uniqueness

#### Unit Test Catches:
```typescript
// ✅ Business logic validation works
it('validates before database call', () => {
  // Mock says user exists
  mockRepository.findByEmail.mockResolvedValue(existingUser);

  // Service should check and reject
  await expect(service.createUser(...)).rejects.toThrow();
});
```

#### What Unit Test MISSES:
- What if repository query is wrong?
- What if database constraint doesn't exist?
- What if the unique index is missing?

#### Integration Test Catches:
```typescript
// ✅ Real database enforces uniqueness
it('database constraint prevents duplicates', async () => {
  await service.createUser({ email: 'test@test.com', ... });

  // Even if business logic failed, database should reject
  await expect(
    service.createUser({ email: 'test@test.com', ... })
  ).rejects.toThrow();

  // Verify actual database state
  const count = await prisma.user.count();
  expect(count).toBe(1);
});
```

---

## Test File Organization

```
tests/
├── unit/
│   ├── users/
│   │   ├── user-service.unit.test.ts       # Service with mocked repo
│   │   └── user-domain.unit.test.ts        # Domain model logic
│   └── posts/
│       ├── create-post-use-case.unit.test.ts  # Use case with mocked repo
│       └── update-post-use-case.unit.test.ts
│
├── integration/
│   ├── users/
│   │   ├── user-repository.integration.test.ts  # Repo + DB only
│   │   └── user-service.integration.test.ts     # Service + Repo + DB
│   └── posts/
│       ├── post-repository.integration.test.ts  # Repo + DB only
│       └── post-use-cases.integration.test.ts   # Use cases + Repo + DB
│
├── api/
│   ├── users/
│   │   └── user-endpoints.api.test.ts      # HTTP + mocked DB
│   └── posts/
│       └── post-endpoints.api.test.ts
│
└── e2e/
    ├── users/
    │   └── user-workflows.e2e.test.ts      # Full stack
    └── posts/
        └── post-workflows.e2e.test.ts
```

---

## When to Write Each Test Type

### Start with Unit Tests ⚡
- Write first during development
- Fast feedback loop
- Test business rules
- Cover edge cases

### Add Integration Tests 🔧
- After unit tests pass
- Verify database schema
- Test data layer
- **Test business logic + database together**
- Validate constraints

### Add API Tests 📡
- Test HTTP contracts
- Verify status codes
- Validate request/response format
- Test authentication/authorization

### Add E2E Tests 🎯
- Test critical user journeys
- Verify complete workflows
- Pre-deployment validation
- Smoke tests

---

## Coverage Goals

```
Unit Tests:        ~70% of tests (fast, many edge cases)
Integration Tests: ~20% of tests (business + data)
  - Repository:    ~10%
  - Service/UC:    ~10%  ← This was missing!
API Tests:         ~7% of tests (HTTP contracts)
E2E Tests:         ~3% of tests (critical paths)
```

---

## The Missing Piece ✨

**Before:** We had unit tests (mocked) and repository integration tests (DB only)

**Gap:** We didn't test Services/Use Cases WITH real database

**Now:** Integration tests cover:
1. Repository ↔ Database
2. Service/Use Case ↔ Repository ↔ Database ✨ **NEW**

This catches bugs where:
- Business logic assumes database behavior
- Validation + constraints interact
- Cascade operations don't work as expected
- Foreign keys are misconfigured
