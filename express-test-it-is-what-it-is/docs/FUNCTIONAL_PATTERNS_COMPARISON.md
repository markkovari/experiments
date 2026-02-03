# Functional Patterns Comparison: HOF vs Direct Functions

This project demonstrates two functional approaches for the Posts module:

1. **Original:** Higher-Order Functions (HOF) with Dependency Injection
2. **Alternative:** Direct Functions with Singleton Pattern

Both are **functional** (not OOP), but they handle dependencies differently.

---

## Side-by-Side Comparison

### Use Case Implementation

#### Original Approach (HOF with DI)
```typescript
// src/modules/posts/use-cases/create-post.ts
import { PostRepository } from '../repository/post-repository';
import { CreatePostInput, Post } from '../domain/types';

export type CreatePostUseCase = (input: CreatePostInput) => Promise<Post>;

// Higher-order function returns the actual use case function
export const createPostUseCase =
  (postRepository: PostRepository): CreatePostUseCase =>
  async (input: CreatePostInput) => {
    // Validation
    if (!input.title || input.title.trim().length === 0) {
      throw new Error('Title is required');
    }

    // Use injected repository
    return postRepository.create(input);
  };
```

**Characteristics:**
- ✅ Dependency injected via function parameter
- ✅ Pure function (no side effects at module level)
- ✅ Easy to test (just pass mock repository)
- ⚠️ Requires currying/HOF pattern
- ⚠️ More verbose

---

#### Alternative Approach (Direct Function with Singleton)
```typescript
// src/modules/posts/use-cases-alternative/create-post.ts
import { prisma } from '../../../shared/prisma/client';
import { createPostRepository } from '../repository/post-repository';
import { CreatePostInput, Post } from '../domain/types';

// Dependency resolved at module level (singleton)
const postRepository = createPostRepository(prisma);

// Direct function
export async function createPost(input: CreatePostInput): Promise<Post> {
  // Validation
  if (!input.title || input.title.trim().length === 0) {
    throw new Error('Title is required');
  }

  // Use module-level repository
  return postRepository.create(input);
}
```

**Characteristics:**
- ✅ Simple, direct function
- ✅ No currying needed
- ✅ Easier to read for beginners
- ⚠️ Harder to test (requires module mocking)
- ⚠️ Creates side effect at module level
- ⚠️ Can't easily swap implementation

---

### Handler Implementation

#### Original Approach (Factory Functions)
```typescript
// src/modules/posts/handlers/post-handlers.ts
import { CreatePostUseCase } from '../use-cases';

export const createCreatePostHandler = (createPost: CreatePostUseCase) => {
  return async (req: Request, res: Response): Promise<void> => {
    try {
      const { title, content, authorId, published } = req.body;
      const post = await createPost({ title, content, authorId, published });
      res.status(201).json(toPostDTO(post));
    } catch (error) {
      // Error handling...
    }
  };
};
```

**Usage in routes:**
```typescript
const createPost = createPostUseCase(postRepository);
const createPostHandler = createCreatePostHandler(createPost);
router.post('/', createPostHandler);
```

---

#### Alternative Approach (Direct Handlers)
```typescript
// src/modules/posts/handlers-alternative/post-handlers.ts
import { createPost } from '../use-cases-alternative';

export async function handleCreatePost(req: Request, res: Response): Promise<void> {
  try {
    const { title, content, authorId, published } = req.body;
    const post = await createPost({ title, content, authorId, published });
    res.status(201).json(toPostDTO(post));
  } catch (error) {
    // Error handling...
  }
}
```

**Usage in routes:**
```typescript
router.post('/', handleCreatePost);
```

---

### Routes Configuration

#### Original Approach
```typescript
// src/modules/posts/routes/post.routes.ts
export function createPostRoutes(prisma: PrismaClient): Router {
  const router = Router();

  // Create repository
  const postRepository = createPostRepository(prisma);

  // Create use cases
  const getAllPosts = getAllPostsUseCase(postRepository);
  const createPost = createPostUseCase(postRepository);
  const updatePost = updatePostUseCase(postRepository);

  // Create handlers
  const getAllPostsHandler = createGetAllPostsHandler(getAllPosts);
  const createPostHandler = createCreatePostHandler(createPost);
  const updatePostHandler = createUpdatePostHandler(updatePost);

  // Define routes
  router.get('/', getAllPostsHandler);
  router.post('/', createPostHandler);
  router.put('/:id', updatePostHandler);

  return router;
}
```

**In app.ts:**
```typescript
app.use('/api/posts', createPostRoutes(prisma));
```

---

#### Alternative Approach
```typescript
// src/modules/posts/routes/post.routes-alternative.ts
export function createPostRoutesAlternative(): Router {
  const router = Router();

  // Define routes with direct handler references
  router.get('/', handleGetAllPosts);
  router.post('/', handleCreatePost);
  router.put('/:id', handleUpdatePost);

  return router;
}
```

**In app.ts:**
```typescript
app.use('/api/posts-alt', createPostRoutesAlternative());
```

---

## Testing Comparison

### Original Approach (Easy to Test)
```typescript
describe('CreatePost Use Case', () => {
  it('should create a post', async () => {
    // Easy: Just create a mock repository
    const mockRepository = {
      create: vi.fn().mockResolvedValue(createdPost),
      // ... other methods
    };

    // Pass mock to use case
    const createPost = createPostUseCase(mockRepository);

    // Test
    const result = await createPost(input);
    expect(result).toEqual(createdPost);
  });
});
```

---

### Alternative Approach (Harder to Test)
```typescript
describe('CreatePost Alternative', () => {
  let createPost: (input: any) => Promise<Post>;
  let mockRepository: any;

  beforeEach(async () => {
    mockRepository = {
      create: vi.fn(),
      // ... other methods
    };

    // Must mock the modules BEFORE importing
    vi.doMock('../repository/post-repository', () => ({
      createPostRepository: vi.fn(() => mockRepository),
    }));

    vi.doMock('../../../shared/prisma/client', () => ({
      prisma: {},
    }));

    // Dynamic import to get mocked version
    const module = await import('../use-cases-alternative/create-post');
    createPost = module.createPost;
  });

  afterEach(() => {
    vi.doUnmock('../repository/post-repository');
    vi.doUnmock('../../../shared/prisma/client');
  });

  it('should create a post', async () => {
    mockRepository.create.mockResolvedValue(createdPost);
    const result = await createPost(input);
    expect(result).toEqual(createdPost);
  });
});
```

---

## Architectural Comparison

```
┌─────────────────────────────────────────────────────────────┐
│                    ORIGINAL APPROACH (HOF)                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Routes (Factory)                                           │
│     ↓ (injects)                                            │
│  Handlers (Factory)                                         │
│     ↓ (injects)                                            │
│  Use Cases (HOF)                                           │
│     ↓ (injects)                                            │
│  Repository (Factory)                                       │
│     ↓ (uses)                                               │
│  Prisma Client                                             │
│                                                             │
│  Dependencies flow DOWN via function parameters            │
│  Everything is pure and composable                         │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                 ALTERNATIVE APPROACH (Direct)               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Routes (Direct)                                            │
│     ↓ (calls)                                              │
│  Handlers (Direct)                                          │
│     ↓ (calls)                                              │
│  Use Cases (Direct)                                         │
│     ↓ (uses singleton)                                     │
│  Repository (Singleton) ← created at module level          │
│     ↓ (uses singleton)                                     │
│  Prisma Client (Singleton) ← imported from shared          │
│                                                             │
│  Dependencies resolved at MODULE LEVEL                      │
│  Simpler but less flexible                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## Trade-offs Summary

| Aspect | Original (HOF) | Alternative (Direct) |
|--------|----------------|---------------------|
| **Complexity** | Higher | Lower |
| **Unit Testing** | ⭐⭐⭐⭐⭐ Excellent | ⭐⭐⭐ Good (needs module mocks) |
| **Integration Testing** | ⭐⭐⭐⭐⭐ Excellent | ⭐ Very Hard (singleton issue) |
| **Flexibility** | ⭐⭐⭐⭐⭐ Easy to swap implementations | ⭐⭐ Harder to swap |
| **Readability** | ⭐⭐⭐ Requires understanding HOF | ⭐⭐⭐⭐⭐ Very clear |
| **Setup Code** | More verbose | Minimal |
| **Type Safety** | ⭐⭐⭐⭐⭐ Excellent | ⭐⭐⭐⭐⭐ Excellent |
| **Performance** | Same (negligible difference) | Same (negligible difference) |
| **Pure Functions** | ✅ Yes | ⚠️ Module-level side effects |
| **Dependency Tracking** | ✅ Explicit in function signatures | ⚠️ Hidden in imports |
| **Integration Tests** | ✅ Easy (inject test DB) | ❌ Hard (singleton with production DB) |

---

## Critical Testing Limitation of Alternative Approach

**⚠️ The singleton pattern makes integration testing difficult!**

### The Problem

```typescript
// Alternative approach creates repository at module level
const postRepository = createPostRepository(prisma); // ← Uses production DB!

export async function createPost(input) {
  return postRepository.create(input); // ← Always uses production repository
}
```

**In integration tests, we need:**
- Test database (testcontainers)
- Isolated test data
- Clean state between tests

**But the singleton means:**
- Repository uses production `prisma` client
- Can't swap for test database
- Module loads before tests can configure it

### Solutions

1. **Skip integration tests for use cases** (recommended)
   - Unit tests: Mock repository ✅
   - E2E tests: Full stack through HTTP ✅
   - Integration tests: Only for repository layer ✅

2. **Mock the entire module** (complex)
   ```typescript
   vi.doMock('../prisma/client', () => ({ prisma: testPrisma }));
   const module = await import('../use-cases-alternative');
   ```

3. **Accept the tradeoff**
   - Simpler code
   - Fewer tests
   - More reliance on E2E tests

### Why HOF Approach is Better for Testing

```typescript
// Original: Easy to inject test database
const testRepository = createPostRepository(testPrisma);
const createPost = createPostUseCase(testRepository);
await createPost(input); // ✅ Uses test database
```

**This is a REAL tradeoff, not just theoretical!**

---

## When to Use Each Approach

### Use Original (HOF with DI) When:
- ✅ Building a large, complex application
- ✅ Need to swap implementations (testing, different environments)
- ✅ Team values functional purity
- ✅ Want maximum testability
- ✅ Building a library/framework

### Use Alternative (Direct Functions) When:
- ✅ Building a small to medium application
- ✅ Team is less familiar with functional patterns
- ✅ Simplicity is more important than flexibility
- ✅ Single implementation is sufficient
- ✅ Rapid prototyping

---

## Try Both APIs

Both approaches are available in this project:

```bash
# Original approach (HOF)
curl http://localhost:3000/api/posts

# Alternative approach (Direct)
curl http://localhost:3000/api/posts-alt
```

Both expose the same API contract, just implemented differently!

---

## Key Takeaway

**Both are functional approaches** - the difference is:
- **HOF:** Dependencies passed as parameters (more flexible, more verbose)
- **Direct:** Dependencies resolved at module level (simpler, less flexible)

Choose based on your project's needs and team's preferences. There's no universally "correct" choice!
