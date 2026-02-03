import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { PrismaClient } from '@prisma/client';
import { createPostRepository } from '../../../src/modules/posts/repository/post-repository';
import { setupTestDatabase, teardownTestDatabase, cleanDatabase } from '../../setup/testcontainers';

describe('PostRepository - Integration Tests', () => {
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

    // Create a test user for post relationships
    const user = await prisma.user.create({
      data: {
        email: 'author@example.com',
        name: 'Test Author',
        password: 'password123',
      },
    });
    testUserId = user.id;
  });

  describe('create', () => {
    it('should create a post in the database', async () => {
      const postData = {
        title: 'Test Post',
        content: 'Test content',
        authorId: testUserId,
        published: false,
      };

      const post = await postRepository.create(postData);

      expect(post.id).toBeDefined();
      expect(post.title).toBe(postData.title);
      expect(post.content).toBe(postData.content);
      expect(post.authorId).toBe(testUserId);
      expect(post.published).toBe(false);
      expect(post.createdAt).toBeInstanceOf(Date);
      expect(post.updatedAt).toBeInstanceOf(Date);
    });

    it('should create a published post', async () => {
      const post = await postRepository.create({
        title: 'Published Post',
        content: 'Content',
        authorId: testUserId,
        published: true,
      });

      expect(post.published).toBe(true);
    });
  });

  describe('findById', () => {
    it('should find a post by id', async () => {
      const createdPost = await postRepository.create({
        title: 'Test Post',
        content: 'Test content',
        authorId: testUserId,
      });

      const foundPost = await postRepository.findById(createdPost.id);

      expect(foundPost).toBeDefined();
      expect(foundPost?.id).toBe(createdPost.id);
      expect(foundPost?.title).toBe('Test Post');
    });

    it('should return null when post not found', async () => {
      const post = await postRepository.findById('non-existent-id');
      expect(post).toBeNull();
    });
  });

  describe('findAll', () => {
    it('should return all posts', async () => {
      await postRepository.create({
        title: 'Post 1',
        content: 'Content 1',
        authorId: testUserId,
      });

      await postRepository.create({
        title: 'Post 2',
        content: 'Content 2',
        authorId: testUserId,
      });

      const posts = await postRepository.findAll();

      expect(posts).toHaveLength(2);
      expect(posts.some((p) => p.title === 'Post 1')).toBe(true);
      expect(posts.some((p) => p.title === 'Post 2')).toBe(true);
    });
  });

  describe('findByAuthorId', () => {
    it('should find posts by author', async () => {
      await postRepository.create({
        title: 'Author Post',
        content: 'Content',
        authorId: testUserId,
      });

      // Create another user and post
      const anotherUser = await prisma.user.create({
        data: {
          email: 'another@example.com',
          name: 'Another User',
          password: 'password123',
        },
      });

      await postRepository.create({
        title: 'Another Post',
        content: 'Content',
        authorId: anotherUser.id,
      });

      const posts = await postRepository.findByAuthorId(testUserId);

      expect(posts).toHaveLength(1);
      expect(posts[0].title).toBe('Author Post');
      expect(posts[0].authorId).toBe(testUserId);
    });
  });

  describe('findPublished', () => {
    it('should return only published posts', async () => {
      await postRepository.create({
        title: 'Published Post',
        content: 'Content',
        authorId: testUserId,
        published: true,
      });

      await postRepository.create({
        title: 'Draft Post',
        content: 'Content',
        authorId: testUserId,
        published: false,
      });

      const posts = await postRepository.findPublished();

      expect(posts).toHaveLength(1);
      expect(posts[0].title).toBe('Published Post');
      expect(posts[0].published).toBe(true);
    });
  });

  describe('update', () => {
    it('should update a post', async () => {
      const createdPost = await postRepository.create({
        title: 'Old Title',
        content: 'Old content',
        authorId: testUserId,
      });

      const updatedPost = await postRepository.update(createdPost.id, {
        title: 'New Title',
        content: 'New content',
      });

      expect(updatedPost.id).toBe(createdPost.id);
      expect(updatedPost.title).toBe('New Title');
      expect(updatedPost.content).toBe('New content');
    });

    it('should publish a post', async () => {
      const createdPost = await postRepository.create({
        title: 'Draft',
        content: 'Content',
        authorId: testUserId,
        published: false,
      });

      const updatedPost = await postRepository.update(createdPost.id, {
        published: true,
      });

      expect(updatedPost.published).toBe(true);
    });
  });

  describe('delete', () => {
    it('should delete a post', async () => {
      const createdPost = await postRepository.create({
        title: 'To Delete',
        content: 'Content',
        authorId: testUserId,
      });

      await postRepository.delete(createdPost.id);

      const foundPost = await postRepository.findById(createdPost.id);
      expect(foundPost).toBeNull();
    });
  });
});
