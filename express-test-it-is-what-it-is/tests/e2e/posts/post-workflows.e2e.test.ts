import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import request from 'supertest';
import { Express } from 'express';
import { PrismaClient } from '@prisma/client';
import { createApp } from '../../../src/app';
import { setupTestDatabase, teardownTestDatabase, cleanDatabase } from '../../setup/testcontainers';

describe('Post Workflows - E2E Tests', () => {
  let app: Express;
  let prisma: PrismaClient;
  let testUserId: string;

  beforeAll(async () => {
    const setup = await setupTestDatabase();
    prisma = setup.prisma;
    app = createApp(prisma);
  }, 60000);

  afterAll(async () => {
    await teardownTestDatabase();
  });

  beforeEach(async () => {
    await cleanDatabase();

    // Create a test user for posts
    const userResponse = await request(app).post('/api/users').send({
      email: 'author@example.com',
      name: 'Test Author',
      password: 'password123',
    });
    testUserId = userResponse.body.id;
  });

  describe('Complete Post CRUD Workflow', () => {
    it('should create, read, update, publish, and delete a post', async () => {
      // Create a draft post
      const createResponse = await request(app).post('/api/posts').send({
        title: 'Test Post',
        content: 'This is test content',
        authorId: testUserId,
        published: false,
      });

      expect(createResponse.status).toBe(201);
      expect(createResponse.body.title).toBe('Test Post');
      expect(createResponse.body.published).toBe(false);
      const postId = createResponse.body.id;

      // Read the post
      const getResponse = await request(app).get(`/api/posts/${postId}`);
      expect(getResponse.status).toBe(200);
      expect(getResponse.body.id).toBe(postId);
      expect(getResponse.body.title).toBe('Test Post');

      // Update the post content
      const updateResponse = await request(app).put(`/api/posts/${postId}`).send({
        title: 'Updated Post Title',
        content: 'Updated content',
      });
      expect(updateResponse.status).toBe(200);
      expect(updateResponse.body.title).toBe('Updated Post Title');
      expect(updateResponse.body.content).toBe('Updated content');

      // Publish the post
      const publishResponse = await request(app).put(`/api/posts/${postId}`).send({
        published: true,
      });
      expect(publishResponse.status).toBe(200);
      expect(publishResponse.body.published).toBe(true);

      // Verify the post is in published list
      const publishedResponse = await request(app).get('/api/posts/published');
      expect(publishedResponse.status).toBe(200);
      expect(publishedResponse.body).toHaveLength(1);
      expect(publishedResponse.body[0].id).toBe(postId);

      // Delete the post
      const deleteResponse = await request(app).delete(`/api/posts/${postId}`);
      expect(deleteResponse.status).toBe(204);

      // Verify deletion
      const getDeletedResponse = await request(app).get(`/api/posts/${postId}`);
      expect(getDeletedResponse.status).toBe(404);
    });
  });

  describe('Author Posts Workflow', () => {
    it('should manage posts by author', async () => {
      // Create another user
      const anotherUserResponse = await request(app).post('/api/users').send({
        email: 'another@example.com',
        name: 'Another Author',
        password: 'password123',
      });
      const anotherUserId = anotherUserResponse.body.id;

      // Create posts for first author
      await request(app).post('/api/posts').send({
        title: 'Author 1 Post 1',
        content: 'Content',
        authorId: testUserId,
      });

      await request(app).post('/api/posts').send({
        title: 'Author 1 Post 2',
        content: 'Content',
        authorId: testUserId,
      });

      // Create post for second author
      await request(app).post('/api/posts').send({
        title: 'Author 2 Post',
        content: 'Content',
        authorId: anotherUserId,
      });

      // Get posts by first author
      const author1Posts = await request(app).get(`/api/posts/author/${testUserId}`);
      expect(author1Posts.status).toBe(200);
      expect(author1Posts.body).toHaveLength(2);

      // Get posts by second author
      const author2Posts = await request(app).get(`/api/posts/author/${anotherUserId}`);
      expect(author2Posts.status).toBe(200);
      expect(author2Posts.body).toHaveLength(1);

      // Get all posts
      const allPosts = await request(app).get('/api/posts');
      expect(allPosts.status).toBe(200);
      expect(allPosts.body).toHaveLength(3);
    });
  });

  describe('Published vs Draft Workflow', () => {
    it('should separate published and draft posts', async () => {
      // Create published posts
      await request(app).post('/api/posts').send({
        title: 'Published Post 1',
        content: 'Content',
        authorId: testUserId,
        published: true,
      });

      await request(app).post('/api/posts').send({
        title: 'Published Post 2',
        content: 'Content',
        authorId: testUserId,
        published: true,
      });

      // Create draft posts
      await request(app).post('/api/posts').send({
        title: 'Draft Post 1',
        content: 'Content',
        authorId: testUserId,
        published: false,
      });

      await request(app).post('/api/posts').send({
        title: 'Draft Post 2',
        content: 'Content',
        authorId: testUserId,
        published: false,
      });

      // Get all posts
      const allPosts = await request(app).get('/api/posts');
      expect(allPosts.body).toHaveLength(4);

      // Get only published posts
      const publishedPosts = await request(app).get('/api/posts/published');
      expect(publishedPosts.body).toHaveLength(2);
      expect(publishedPosts.body.every((p: any) => p.published === true)).toBe(true);
    });
  });

  describe('Post with User Relationship Workflow', () => {
    it('should cascade delete posts when user is deleted', async () => {
      // Create posts for the user
      const post1 = await request(app).post('/api/posts').send({
        title: 'Post 1',
        content: 'Content',
        authorId: testUserId,
      });

      const post2 = await request(app).post('/api/posts').send({
        title: 'Post 2',
        content: 'Content',
        authorId: testUserId,
      });

      // Verify posts exist
      const postsBeforeDelete = await request(app).get(`/api/posts/author/${testUserId}`);
      expect(postsBeforeDelete.body).toHaveLength(2);

      // Delete the user
      await request(app).delete(`/api/users/${testUserId}`);

      // Verify posts are also deleted (cascade)
      const post1Check = await request(app).get(`/api/posts/${post1.body.id}`);
      expect(post1Check.status).toBe(404);

      const post2Check = await request(app).get(`/api/posts/${post2.body.id}`);
      expect(post2Check.status).toBe(404);
    });
  });

  describe('Validation Workflow', () => {
    it('should validate required fields', async () => {
      // Try to create post without title
      const noTitleResponse = await request(app).post('/api/posts').send({
        content: 'Content',
        authorId: testUserId,
      });
      expect(noTitleResponse.status).toBe(400);

      // Try to create post without content
      const noContentResponse = await request(app).post('/api/posts').send({
        title: 'Title',
        authorId: testUserId,
      });
      expect(noContentResponse.status).toBe(400);

      // Try to create post without authorId
      const noAuthorResponse = await request(app).post('/api/posts').send({
        title: 'Title',
        content: 'Content',
      });
      expect(noAuthorResponse.status).toBe(400);
    });
  });
});
