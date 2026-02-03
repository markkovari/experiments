import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import express, { Express } from 'express';
import { UserController } from '../../../src/modules/users/controllers/UserController';
import { UserService } from '../../../src/modules/users/services/UserService';
import { createUserRoutes } from '../../../src/modules/users/routes/user.routes';
import { User } from '../../../src/modules/users/domain/User';

describe('User API Endpoints - API Tests', () => {
  let app: Express;
  let mockUserService: UserService;

  beforeEach(() => {
    // Mock the UserService
    mockUserService = {
      getUserById: vi.fn(),
      getUserByEmail: vi.fn(),
      getAllUsers: vi.fn(),
      createUser: vi.fn(),
      updateUser: vi.fn(),
      deleteUser: vi.fn(),
    } as any;

    // Create Express app with mocked service
    app = express();
    app.use(express.json());

    const userController = new UserController(mockUserService);
    app.use('/api/users', createUserRoutes(userController));
  });

  describe('GET /api/users', () => {
    it('should return all users', async () => {
      const users = [
        User.create({ id: '1', email: 'user1@example.com', name: 'User 1', password: 'pass' }),
        User.create({ id: '2', email: 'user2@example.com', name: 'User 2', password: 'pass' }),
      ];

      vi.mocked(mockUserService.getAllUsers).mockResolvedValue(users);

      const response = await request(app).get('/api/users');

      expect(response.status).toBe(200);
      expect(response.body).toHaveLength(2);
      expect(response.body[0]).not.toHaveProperty('password');
      expect(response.body[0].email).toBe('user1@example.com');
    });

    it('should return empty array when no users', async () => {
      vi.mocked(mockUserService.getAllUsers).mockResolvedValue([]);

      const response = await request(app).get('/api/users');

      expect(response.status).toBe(200);
      expect(response.body).toEqual([]);
    });
  });

  describe('GET /api/users/:id', () => {
    it('should return a user by id', async () => {
      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      vi.mocked(mockUserService.getUserById).mockResolvedValue(user);

      const response = await request(app).get('/api/users/1');

      expect(response.status).toBe(200);
      expect(response.body.id).toBe('1');
      expect(response.body.email).toBe('test@example.com');
      expect(response.body).not.toHaveProperty('password');
    });

    it('should return 404 when user not found', async () => {
      vi.mocked(mockUserService.getUserById).mockResolvedValue(null);

      const response = await request(app).get('/api/users/999');

      expect(response.status).toBe(404);
      expect(response.body.error).toBe('User not found');
    });
  });

  describe('POST /api/users', () => {
    it('should create a new user', async () => {
      const newUser = User.create({
        id: '1',
        email: 'new@example.com',
        name: 'New User',
        password: 'password123',
      });

      vi.mocked(mockUserService.createUser).mockResolvedValue(newUser);

      const response = await request(app).post('/api/users').send({
        email: 'new@example.com',
        name: 'New User',
        password: 'password123',
      });

      expect(response.status).toBe(201);
      expect(response.body.email).toBe('new@example.com');
      expect(response.body).not.toHaveProperty('password');
    });

    it('should return 400 when missing required fields', async () => {
      const response = await request(app).post('/api/users').send({
        email: 'test@example.com',
        // missing name and password
      });

      expect(response.status).toBe(400);
      expect(response.body.error).toBe('Missing required fields');
    });

    it('should return 409 when email already exists', async () => {
      vi.mocked(mockUserService.createUser).mockRejectedValue(
        new Error('User with this email already exists')
      );

      const response = await request(app).post('/api/users').send({
        email: 'existing@example.com',
        name: 'Test User',
        password: 'password123',
      });

      expect(response.status).toBe(409);
      expect(response.body.error).toBe('User with this email already exists');
    });
  });

  describe('PUT /api/users/:id', () => {
    it('should update a user', async () => {
      const updatedUser = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Updated Name',
        password: 'password123',
      });

      vi.mocked(mockUserService.updateUser).mockResolvedValue(updatedUser);

      const response = await request(app).put('/api/users/1').send({
        name: 'Updated Name',
      });

      expect(response.status).toBe(200);
      expect(response.body.name).toBe('Updated Name');
    });

    it('should return 404 when user not found', async () => {
      vi.mocked(mockUserService.updateUser).mockRejectedValue(new Error('User not found'));

      const response = await request(app).put('/api/users/999').send({
        name: 'New Name',
      });

      expect(response.status).toBe(404);
      expect(response.body.error).toBe('User not found');
    });
  });

  describe('DELETE /api/users/:id', () => {
    it('should delete a user', async () => {
      vi.mocked(mockUserService.deleteUser).mockResolvedValue(undefined);

      const response = await request(app).delete('/api/users/1');

      expect(response.status).toBe(204);
    });

    it('should return 404 when user not found', async () => {
      vi.mocked(mockUserService.deleteUser).mockRejectedValue(new Error('User not found'));

      const response = await request(app).delete('/api/users/999');

      expect(response.status).toBe(404);
      expect(response.body.error).toBe('User not found');
    });
  });
});
