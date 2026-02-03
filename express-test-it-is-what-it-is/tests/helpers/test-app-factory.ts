import express, { Express, Router } from 'express';
import { PrismaClient } from '@prisma/client';

/**
 * Test App Factory - Compose minimal Express apps for testing
 *
 * Instead of loading the entire app, build exactly what you need:
 * - Only the routes you're testing
 * - Only the middleware you need
 * - Mock or real dependencies
 */

export type TestAppOptions = {
  // Middleware
  json?: boolean;
  urlencoded?: boolean;
  customMiddleware?: any[];

  // Routes
  routes?: { path: string; router: Router }[];

  // Error handling
  notFoundHandler?: boolean;
  errorHandler?: boolean;

  // Dependencies
  prisma?: PrismaClient;
};

/**
 * Create a minimal Express app for testing
 */
export function createTestApp(options: TestAppOptions = {}): Express {
  const app = express();

  // Add middleware only if requested
  if (options.json !== false) {
    app.use(express.json());
  }

  if (options.urlencoded !== false) {
    app.use(express.urlencoded({ extended: true }));
  }

  if (options.customMiddleware) {
    options.customMiddleware.forEach((middleware) => app.use(middleware));
  }

  // Add only the routes specified
  if (options.routes) {
    options.routes.forEach(({ path, router }) => {
      app.use(path, router);
    });
  }

  // Optional 404 handler
  if (options.notFoundHandler !== false) {
    app.use((req, res) => {
      res.status(404).json({ error: 'Not found' });
    });
  }

  // Optional error handler
  if (options.errorHandler) {
    app.use((err: any, req: any, res: any, next: any) => {
      res.status(500).json({ error: err.message });
    });
  }

  return app;
}

/**
 * Quick builder for single-route tests
 */
export function createSingleRouteApp(path: string, router: Router): Express {
  return createTestApp({
    routes: [{ path, router }],
  });
}

/**
 * Builder for testing without any middleware
 */
export function createMinimalApp(routes: { path: string; router: Router }[]): Express {
  return createTestApp({
    json: false,
    urlencoded: false,
    notFoundHandler: false,
    routes,
  });
}
