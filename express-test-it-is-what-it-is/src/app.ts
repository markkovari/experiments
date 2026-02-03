import express, { Express, Request, Response } from 'express';
import { PrismaClient } from '@prisma/client';
import { UserRepository } from './modules/users/repository/UserRepository';
import { UserService } from './modules/users/services/UserService';
import { UserController } from './modules/users/controllers/UserController';
import { createUserRoutes } from './modules/users/routes/user.routes';
import { createPostRoutes } from './modules/posts/routes/post.routes';
import { createPostRoutesAlternative } from './modules/posts/routes/post.routes-alternative';

export function createApp(prisma: PrismaClient): Express {
  const app = express();

  // Middleware
  app.use(express.json());
  app.use(express.urlencoded({ extended: true }));

  // Health check
  app.get('/health', (req: Request, res: Response) => {
    res.json({ status: 'ok', timestamp: new Date().toISOString() });
  });

  // Users routes (OOP style)
  const userRepository = new UserRepository(prisma);
  const userService = new UserService(userRepository);
  const userController = new UserController(userService);
  app.use('/api/users', createUserRoutes(userController));

  // Posts routes (Functional style - Higher-Order Functions with dependency injection)
  app.use('/api/posts', createPostRoutes(prisma));

  // Posts routes (Alternative - Direct functions with singleton repository)
  app.use('/api/posts-alt', createPostRoutesAlternative());

  // 404 handler
  app.use((req: Request, res: Response) => {
    res.status(404).json({ error: 'Not found' });
  });

  return app;
}
