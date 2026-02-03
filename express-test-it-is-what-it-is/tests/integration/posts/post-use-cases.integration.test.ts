import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { PrismaClient } from '@prisma/client';
import { createPostRepository } from '../../../src/modules/posts/repository/post-repository';
import {
  createPostUseCase,
  getPostByIdUseCase,
  getAllPostsUseCase,
  getPublishedPostsUseCase,
  updatePostUseCase,
  deletePostUseCase,
} from '../../../src/modules/posts/use-cases';
import { setupTestDatabase, teardownTestDatabase, cleanDatabase } from '../../setup/testcontainers';

describe('Post Use Cases - Integration Tests', () => {
  let prisma: PrismaClient;
  let postRepository: ReturnType<typeof createPostRepository>;
  let testUserId: string;

  beforeAll(async () => {
    const setup = await setupTestDatabase();
    prisma = setup.prisma;
    postRepository = createPostRepository(prisma);
  }, 60000);

  afterAll(async () => {
    await teardownTestDatabase();
  });

  beforeEach(async () => {
    await cleanDatabase();

    // Create test user
    const user = await prisma.user.create({
      data: {
        email: 'author@example.com',
        name: 'Test Author',
        password: 'password123',
      },
    });
    testUserId = user.id;
  });

  describe('createPostUseCase with real database', () => {
    it('should create post and persist to database', async () => {
      const createPost = createPostUseCase(postRepository);

      const post = await createPost({
        title: 'Integration Test Post',
        content: 'This is content from integration test',
        authorId: testUserId,
        published: false,
      });

      expect(post.id).toBeDefined();
      expect(post.title).toBe('Integration Test Post');

      // Verify it was actually saved to database
      const getPost = getPostByIdUseCase(postRepository);
      const retrieved = await getPost(post.id);

      expect(retrieved).not.toBeNull();
      expect(retrieved?.title).toBe('Integration Test Post');
    });

    it('should validate input before database operation', async () => {
      const createPost = createPostUseCase(postRepository);

      // Empty title should fail validation before hitting database
      await expect(
        createPost({
          title: '',
          content: 'Content',
          authorId: testUserId,
        })
      ).rejects.toThrow('Title is required');

      // Verify no post was created in database
      const getAllPosts = getAllPostsUseCase(postRepository);
      const allPosts = await getAllPosts();
      expect(allPosts).toHaveLength(0);
    });

    it('should enforce foreign key constraint', async () => {
      const createPost = createPostUseCase(postRepository);

      // Try to create post with non-existent author
      // This should fail at database level with foreign key constraint
      await expect(
        createPost({
          title: 'Post',
          content: 'Content',
          authorId: 'non-existent-user-id',
        })
      ).rejects.toThrow();
    });
  });

  describe('updatePostUseCase with real database', () => {
    it('should update post and persist changes', async () => {
      // Create a post first
      const createPost = createPostUseCase(postRepository);
      const created = await createPost({
        title: 'Original Title',
        content: 'Original Content',
        authorId: testUserId,
      });

      // Update the post
      const updatePost = updatePostUseCase(postRepository);
      const updated = await updatePost(created.id, {
        title: 'Updated Title',
        published: true,
      });

      expect(updated.title).toBe('Updated Title');
      expect(updated.published).toBe(true);
      expect(updated.content).toBe('Original Content'); // Should remain unchanged

      // Verify in database
      const getPost = getPostByIdUseCase(postRepository);
      const fromDb = await getPost(created.id);

      expect(fromDb?.title).toBe('Updated Title');
      expect(fromDb?.published).toBe(true);
    });

    it('should handle non-existent post', async () => {
      const updatePost = updatePostUseCase(postRepository);

      await expect(
        updatePost('non-existent-id', { title: 'New Title' })
      ).rejects.toThrow('Post not found');
    });

    it('should validate updates before database operation', async () => {
      const createPost = createPostUseCase(postRepository);
      const post = await createPost({
        title: 'Test Post',
        content: 'Content',
        authorId: testUserId,
      });

      const updatePost = updatePostUseCase(postRepository);

      // Empty title should fail validation
      await expect(updatePost(post.id, { title: '' })).rejects.toThrow('Title cannot be empty');

      // Verify post was not updated in database
      const getPost = getPostByIdUseCase(postRepository);
      const fromDb = await getPost(post.id);
      expect(fromDb?.title).toBe('Test Post'); // Original title
    });
  });

  describe('deletePostUseCase with real database', () => {
    it('should delete post and verify removal', async () => {
      // Create post
      const createPost = createPostUseCase(postRepository);
      const post = await createPost({
        title: 'To Delete',
        content: 'Content',
        authorId: testUserId,
      });

      // Verify it exists
      const getPost = getPostByIdUseCase(postRepository);
      const beforeDelete = await getPost(post.id);
      expect(beforeDelete).not.toBeNull();

      // Delete post
      const deletePost = deletePostUseCase(postRepository);
      await deletePost(post.id);

      // Verify deletion
      const afterDelete = await getPost(post.id);
      expect(afterDelete).toBeNull();
    });

    it('should handle deleting non-existent post', async () => {
      const deletePost = deletePostUseCase(postRepository);

      await expect(deletePost('non-existent-id')).rejects.toThrow('Post not found');
    });
  });

  describe('getPublishedPostsUseCase with real database', () => {
    it('should filter published posts correctly', async () => {
      const createPost = createPostUseCase(postRepository);

      // Create mix of published and draft posts
      await createPost({
        title: 'Published 1',
        content: 'Content',
        authorId: testUserId,
        published: true,
      });

      await createPost({
        title: 'Draft 1',
        content: 'Content',
        authorId: testUserId,
        published: false,
      });

      await createPost({
        title: 'Published 2',
        content: 'Content',
        authorId: testUserId,
        published: true,
      });

      await createPost({
        title: 'Draft 2',
        content: 'Content',
        authorId: testUserId,
        published: false,
      });

      // Get only published posts
      const getPublished = getPublishedPostsUseCase(postRepository);
      const published = await getPublished();

      expect(published).toHaveLength(2);
      expect(published.every((p) => p.published === true)).toBe(true);
      expect(published.some((p) => p.title === 'Published 1')).toBe(true);
      expect(published.some((p) => p.title === 'Published 2')).toBe(true);
    });
  });

  describe('complex scenarios with cascade delete', () => {
    it('should cascade delete posts when author is deleted', async () => {
      const createPost = createPostUseCase(postRepository);

      // Create posts for author
      await createPost({ title: 'Post 1', content: 'Content', authorId: testUserId });
      await createPost({ title: 'Post 2', content: 'Content', authorId: testUserId });

      // Verify posts exist
      const getAllPosts = getAllPostsUseCase(postRepository);
      const beforeDelete = await getAllPosts();
      expect(beforeDelete).toHaveLength(2);

      // Delete the author (should cascade to posts)
      await prisma.user.delete({ where: { id: testUserId } });

      // Verify posts were deleted
      const afterDelete = await getAllPosts();
      expect(afterDelete).toHaveLength(0);
    });
  });

  describe('transaction-like behavior', () => {
    it('should maintain consistency across multiple operations', async () => {
      const createPost = createPostUseCase(postRepository);
      const updatePost = updatePostUseCase(postRepository);

      // Create post
      const post = await createPost({
        title: 'Initial',
        content: 'Content',
        authorId: testUserId,
        published: false,
      });

      // Multiple updates
      await updatePost(post.id, { title: 'Updated 1' });
      await updatePost(post.id, { title: 'Updated 2' });
      await updatePost(post.id, { published: true });

      // Final state should be consistent
      const getPost = getPostByIdUseCase(postRepository);
      const final = await getPost(post.id);

      expect(final?.title).toBe('Updated 2');
      expect(final?.published).toBe(true);
      expect(final?.content).toBe('Content'); // Unchanged
    });
  });
});
