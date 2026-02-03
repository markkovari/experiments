import { describe, it, expect, vi, beforeEach } from 'vitest';
import request from 'supertest';
import { UserController } from '../../../src/modules/users/controllers/UserController';
import { UserService } from '../../../src/modules/users/services/UserService';
import { createUserRoutes } from '../../../src/modules/users/routes/user.routes';
import { User } from '../../../src/modules/users/domain/User';
import { createTestApp, createSingleRouteApp } from '../../helpers/test-app-factory';

/**
 * Focused API Tests - Using Test App Factory
 *
 * Benefits:
 * - Only load the routes you're testing
 * - Explicit about what middleware is used
 * - Faster test execution
 * - Better isolation
 */

describe('User API Endpoints - Focused Tests', () => {
  let mockUserService: UserService;

  beforeEach(() => {
    mockUserService = {
      getUserById: vi.fn(),
      getUserByEmail: vi.fn(),
      getAllUsers: vi.fn(),
      createUser: vi.fn(),
      updateUser: vi.fn(),
      deleteUser: vi.fn(),
    } as any;
  });

  describe('GET /api/users/:id - Minimal setup', () => {
    it('should return user by id', async () => {
      // Arrange: Create app with ONLY the user routes
      const userController = new UserController(mockUserService);
      const app = createSingleRouteApp('/api/users', createUserRoutes(userController));

      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      vi.mocked(mockUserService.getUserById).mockResolvedValue(user);

      // Act & Assert
      const response = await request(app).get('/api/users/1');

      expect(response.status).toBe(200);
      expect(response.body.id).toBe('1');
      expect(response.body.email).toBe('test@example.com');
      expect(response.body).not.toHaveProperty('password');
    });

    it('should return 404 when user not found', async () => {
      const userController = new UserController(mockUserService);
      const app = createSingleRouteApp('/api/users', createUserRoutes(userController));

      vi.mocked(mockUserService.getUserById).mockResolvedValue(null);

      const response = await request(app).get('/api/users/999');

      expect(response.status).toBe(404);
      expect(response.body.error).toBe('User not found');
    });
  });

  describe('POST /api/users - With custom middleware', () => {
    it('should create user with request logging', async () => {
      // Custom middleware for this test
      const requestLogger = vi.fn((req, res, next) => next());

      const userController = new UserController(mockUserService);
      const app = createTestApp({
        customMiddleware: [requestLogger],
        routes: [{ path: '/api/users', router: createUserRoutes(userController) }],
      });

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
      expect(requestLogger).toHaveBeenCalled();
    });
  });

  describe('Multiple routes - Composed app', () => {
    it('should handle users and health check only', async () => {
      const userController = new UserController(mockUserService);

      // Build app with ONLY users + health check
      const app = createTestApp({
        routes: [
          { path: '/api/users', router: createUserRoutes(userController) },
          {
            path: '/health',
            router: (() => {
              const router = require('express').Router();
              router.get('/', (req: any, res: any) => res.json({ status: 'ok' }));
              return router;
            })(),
          },
        ],
      });

      vi.mocked(mockUserService.getAllUsers).mockResolvedValue([]);

      // Test users endpoint
      const usersResponse = await request(app).get('/api/users');
      expect(usersResponse.status).toBe(200);

      // Test health endpoint
      const healthResponse = await request(app).get('/health');
      expect(healthResponse.status).toBe(200);
      expect(healthResponse.body.status).toBe('ok');

      // Other routes should 404
      const postsResponse = await request(app).get('/api/posts');
      expect(postsResponse.status).toBe(404);
    });
  });

  describe('Without middleware - Raw testing', () => {
    it('should test handler without JSON parsing', async () => {
      const userController = new UserController(mockUserService);

      // App with NO middleware
      const app = createTestApp({
        json: false,
        urlencoded: false,
        notFoundHandler: false,
        routes: [{ path: '/api/users', router: createUserRoutes(userController) }],
      });

      vi.mocked(mockUserService.getAllUsers).mockResolvedValue([]);

      const response = await request(app).get('/api/users');

      expect(response.status).toBe(200);
      // Response is raw, not parsed as JSON by Express
    });
  });
});
