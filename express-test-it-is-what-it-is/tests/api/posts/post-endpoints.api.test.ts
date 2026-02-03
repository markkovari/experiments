import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import express, { Express } from 'express';
import { PrismaClient } from '@prisma/client';
import { createPostRoutes } from '../../../src/modules/posts/routes/post.routes';
import { Post } from '../../../src/modules/posts/domain/types';

describe('Post API Endpoints - API Tests', () => {
  let app: Express;
  let mockPrisma: any;

  beforeEach(() => {
    // Mock Prisma Client
    mockPrisma = {
      post: {
        findUnique: vi.fn(),
        findMany: vi.fn(),
        create: vi.fn(),
        update: vi.fn(),
        delete: vi.fn(),
      },
    };

    // Create Express app with mocked Prisma
    app = express();
    app.use(express.json());
    app.use('/api/posts', createPostRoutes(mockPrisma as PrismaClient));
  });

  describe('GET /api/posts', () => {
    it('should return all posts', async () => {
      const posts = [
        {
          id: '1',
          title: 'Post 1',
          content: 'Content 1',
          published: true,
          authorId: 'user-1',
          createdAt: new Date(),
          updatedAt: new Date(),
        },
        {
          id: '2',
          title: 'Post 2',
          content: 'Content 2',
          published: false,
          authorId: 'user-1',
          createdAt: new Date(),
          updatedAt: new Date(),
        },
      ];

      mockPrisma.post.findMany.mockResolvedValue(posts);

      const response = await request(app).get('/api/posts');

      expect(response.status).toBe(200);
      expect(response.body).toHaveLength(2);
      expect(response.body[0].title).toBe('Post 1');
    });
  });

  describe('GET /api/posts/published', () => {
    it('should return only published posts', async () => {
      const posts = [
        {
          id: '1',
          title: 'Published Post',
          content: 'Content',
          published: true,
          authorId: 'user-1',
          createdAt: new Date(),
          updatedAt: new Date(),
        },
      ];

      mockPrisma.post.findMany.mockResolvedValue(posts);

      const response = await request(app).get('/api/posts/published');

      expect(response.status).toBe(200);
      expect(response.body).toHaveLength(1);
      expect(response.body[0].published).toBe(true);
    });
  });

  describe('GET /api/posts/:id', () => {
    it('should return a post by id', async () => {
      const post = {
        id: '1',
        title: 'Test Post',
        content: 'Test content',
        published: true,
        authorId: 'user-1',
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      mockPrisma.post.findUnique.mockResolvedValue(post);

      const response = await request(app).get('/api/posts/1');

      expect(response.status).toBe(200);
      expect(response.body.id).toBe('1');
      expect(response.body.title).toBe('Test Post');
    });

    it('should return 404 when post not found', async () => {
      mockPrisma.post.findUnique.mockResolvedValue(null);

      const response = await request(app).get('/api/posts/999');

      expect(response.status).toBe(404);
      expect(response.body.error).toBe('Post not found');
    });
  });

  describe('GET /api/posts/author/:authorId', () => {
    it('should return posts by author', async () => {
      const posts = [
        {
          id: '1',
          title: 'Author Post',
          content: 'Content',
          published: true,
          authorId: 'user-1',
          createdAt: new Date(),
          updatedAt: new Date(),
        },
      ];

      mockPrisma.post.findMany.mockResolvedValue(posts);

      const response = await request(app).get('/api/posts/author/user-1');

      expect(response.status).toBe(200);
      expect(response.body).toHaveLength(1);
      expect(response.body[0].authorId).toBe('user-1');
    });
  });

  describe('POST /api/posts', () => {
    it('should create a new post', async () => {
      const newPost = {
        id: '1',
        title: 'New Post',
        content: 'New content',
        published: false,
        authorId: 'user-1',
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      mockPrisma.post.create.mockResolvedValue(newPost);

      const response = await request(app).post('/api/posts').send({
        title: 'New Post',
        content: 'New content',
        authorId: 'user-1',
      });

      expect(response.status).toBe(201);
      expect(response.body.title).toBe('New Post');
    });

    it('should return 400 when title is empty', async () => {
      const response = await request(app).post('/api/posts').send({
        title: '',
        content: 'Content',
        authorId: 'user-1',
      });

      expect(response.status).toBe(400);
      expect(response.body.error).toBe('Title is required');
    });

    it('should return 400 when content is missing', async () => {
      const response = await request(app).post('/api/posts').send({
        title: 'Title',
        content: '',
        authorId: 'user-1',
      });

      expect(response.status).toBe(400);
      expect(response.body.error).toBe('Content is required');
    });
  });

  describe('PUT /api/posts/:id', () => {
    it('should update a post', async () => {
      const existingPost = {
        id: '1',
        title: 'Old Title',
        content: 'Old content',
        published: false,
        authorId: 'user-1',
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      const updatedPost = {
        ...existingPost,
        title: 'New Title',
      };

      mockPrisma.post.findUnique.mockResolvedValue(existingPost);
      mockPrisma.post.update.mockResolvedValue(updatedPost);

      const response = await request(app).put('/api/posts/1').send({
        title: 'New Title',
      });

      expect(response.status).toBe(200);
      expect(response.body.title).toBe('New Title');
    });

    it('should return 404 when post not found', async () => {
      mockPrisma.post.findUnique.mockResolvedValue(null);

      const response = await request(app).put('/api/posts/999').send({
        title: 'New Title',
      });

      expect(response.status).toBe(404);
      expect(response.body.error).toBe('Post not found');
    });
  });

  describe('DELETE /api/posts/:id', () => {
    it('should delete a post', async () => {
      const post = {
        id: '1',
        title: 'To Delete',
        content: 'Content',
        published: false,
        authorId: 'user-1',
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      mockPrisma.post.findUnique.mockResolvedValue(post);
      mockPrisma.post.delete.mockResolvedValue(post);

      const response = await request(app).delete('/api/posts/1');

      expect(response.status).toBe(204);
    });

    it('should return 404 when post not found', async () => {
      mockPrisma.post.findUnique.mockResolvedValue(null);

      const response = await request(app).delete('/api/posts/999');

      expect(response.status).toBe(404);
      expect(response.body.error).toBe('Post not found');
    });
  });
});
