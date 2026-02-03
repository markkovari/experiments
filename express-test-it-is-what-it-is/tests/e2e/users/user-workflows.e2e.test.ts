import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import request from 'supertest';
import { Express } from 'express';
import { PrismaClient } from '@prisma/client';
import { createApp } from '../../../src/app';
import { setupTestDatabase, teardownTestDatabase, cleanDatabase } from '../../setup/testcontainers';

describe('User Workflows - E2E Tests', () => {
  let app: Express;
  let prisma: PrismaClient;

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
  });

  describe('Complete User CRUD Workflow', () => {
    it('should create, read, update, and delete a user', async () => {
      // Create a user
      const createResponse = await request(app).post('/api/users').send({
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      expect(createResponse.status).toBe(201);
      expect(createResponse.body.email).toBe('test@example.com');
      expect(createResponse.body).not.toHaveProperty('password');
      const userId = createResponse.body.id;

      // Read the user
      const getResponse = await request(app).get(`/api/users/${userId}`);
      expect(getResponse.status).toBe(200);
      expect(getResponse.body.id).toBe(userId);
      expect(getResponse.body.email).toBe('test@example.com');

      // Update the user
      const updateResponse = await request(app).put(`/api/users/${userId}`).send({
        name: 'Updated Name',
      });
      expect(updateResponse.status).toBe(200);
      expect(updateResponse.body.name).toBe('Updated Name');
      expect(updateResponse.body.email).toBe('test@example.com');

      // Verify the update
      const getUpdatedResponse = await request(app).get(`/api/users/${userId}`);
      expect(getUpdatedResponse.body.name).toBe('Updated Name');

      // Delete the user
      const deleteResponse = await request(app).delete(`/api/users/${userId}`);
      expect(deleteResponse.status).toBe(204);

      // Verify deletion
      const getDeletedResponse = await request(app).get(`/api/users/${userId}`);
      expect(getDeletedResponse.status).toBe(404);
    });
  });

  describe('Multiple Users Workflow', () => {
    it('should handle multiple users', async () => {
      // Create multiple users
      const user1Response = await request(app).post('/api/users').send({
        email: 'user1@example.com',
        name: 'User 1',
        password: 'password123',
      });
      expect(user1Response.status).toBe(201);

      const user2Response = await request(app).post('/api/users').send({
        email: 'user2@example.com',
        name: 'User 2',
        password: 'password123',
      });
      expect(user2Response.status).toBe(201);

      // Get all users
      const getAllResponse = await request(app).get('/api/users');
      expect(getAllResponse.status).toBe(200);
      expect(getAllResponse.body).toHaveLength(2);

      // Verify both users are present
      const emails = getAllResponse.body.map((u: any) => u.email);
      expect(emails).toContain('user1@example.com');
      expect(emails).toContain('user2@example.com');
    });
  });

  describe('Email Uniqueness Workflow', () => {
    it('should prevent duplicate emails', async () => {
      // Create first user
      const firstResponse = await request(app).post('/api/users').send({
        email: 'unique@example.com',
        name: 'First User',
        password: 'password123',
      });
      expect(firstResponse.status).toBe(201);

      // Try to create second user with same email
      const secondResponse = await request(app).post('/api/users').send({
        email: 'unique@example.com',
        name: 'Second User',
        password: 'password123',
      });
      expect(secondResponse.status).toBe(409);
      expect(secondResponse.body.error).toBe('User with this email already exists');
    });

    it('should prevent email update to existing email', async () => {
      // Create two users
      const user1 = await request(app).post('/api/users').send({
        email: 'user1@example.com',
        name: 'User 1',
        password: 'password123',
      });

      const user2 = await request(app).post('/api/users').send({
        email: 'user2@example.com',
        name: 'User 2',
        password: 'password123',
      });

      // Try to update user2's email to user1's email
      const updateResponse = await request(app)
        .put(`/api/users/${user2.body.id}`)
        .send({
          email: 'user1@example.com',
        });

      expect(updateResponse.status).toBe(409);
      expect(updateResponse.body.error).toBe('Email already in use');
    });
  });

  describe('Validation Workflow', () => {
    it('should validate required fields on creation', async () => {
      const response = await request(app).post('/api/users').send({
        email: 'test@example.com',
        // missing name and password
      });

      expect(response.status).toBe(400);
      expect(response.body.error).toBe('Missing required fields');
    });
  });
});
