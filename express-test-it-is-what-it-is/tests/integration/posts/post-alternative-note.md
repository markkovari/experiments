# Note on Testing Alternative Approach

## The Challenge with Singleton Pattern

The alternative approach uses a **singleton repository** created at module level:

```typescript
// src/modules/posts/use-cases-alternative/create-post.ts
import { prisma } from '../../../shared/prisma/client';
import { createPostRepository } from '../repository/post-repository';

// Created once at module load time
const postRepository = createPostRepository(prisma);

export async function createPost(input: CreatePostInput): Promise<Post> {
  return postRepository.create(input);
}
```

## Why Integration Tests Are Difficult

**Problem:** The repository uses the production `prisma` client, not the test database.

**In tests, we need:**
- TestContainers PostgreSQL
- Test database with migrations
- Isolated test data

**But the singleton pattern means:**
- Repository is created with production prisma client
- Can't swap it for test database
- Module-level side effects happen before tests run

## Solutions

### Option 1: Mock the Module (Complex)
```typescript
// Need to mock BEFORE importing
vi.doMock('../../../src/shared/prisma/client', () => ({
  prisma: testPrismaClient
}));

const module = await import('../use-cases-alternative/create-post');
// Now it uses test database
```

**Drawbacks:**
- Complex setup
- Must mock every test file
- Fragile

### Option 2: Don't Write Integration Tests for This Approach
**Recommendation:**
- Use **unit tests** with mocked repository (easier)
- Use **E2E tests** for full stack validation
- Skip integration tests for use cases

**Why this works:**
- Unit tests: Mock repository easily
- E2E tests: Use real app, real database through HTTP
- Integration tests: Only for repository layer (works fine)

### Option 3: Use a Test-Specific Entry Point
Create test-specific versions that accept dependencies:

```typescript
// For testing only
export async function createPostWithRepository(
  repository: PostRepository,
  input: CreatePostInput
): Promise<Post> {
  // Same logic, but uses injected repository
}
```

**Drawbacks:**
- Maintains two versions
- Defeats the purpose of the alternative approach

## Comparison with Original Approach

**Original (HOF with DI):**
```typescript
// ✅ Easy to test
const createPost = createPostUseCase(testRepository);
await createPost(input);
```

**Alternative (Singleton):**
```typescript
// ❌ Hard to test with real database
// Must mock modules or skip integration tests
await createPost(input);
```

## Conclusion

This is **a real tradeoff** of the singleton pattern:
- ✅ Simpler code
- ✅ Easier to read
- ❌ Harder to test with different dependencies
- ❌ Can't swap implementations

For this reason, **the original HOF approach is recommended** when:
- Testability is important
- Need to swap implementations
- Building complex systems

The alternative approach is fine when:
- Simple applications
- Can rely on unit tests + E2E tests
- Don't need to swap implementations
