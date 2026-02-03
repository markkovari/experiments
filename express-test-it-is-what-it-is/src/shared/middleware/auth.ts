import { Request, Response, NextFunction } from 'express';
import { UnauthorizedError } from './error-handler';
import { AuthUser } from '../types/express-helpers';

/**
 * Mock authentication middleware
 *
 * In a real app, this would:
 * 1. Extract token from Authorization header
 * 2. Verify JWT token
 * 3. Load user from database
 * 4. Attach user to request
 *
 * For demo purposes, it:
 * - Checks for Authorization header
 * - Creates a mock user
 * - Attaches to request
 *
 * After this middleware runs, handlers can use AuthenticatedRequestHandler type
 * and TypeScript will know that req.user exists
 *
 * Usage:
 * router.get('/profile', requireAuth, getProfileHandler);
 */
export const requireAuth = async (req: Request, res: Response, next: NextFunction) => {
  try {
    const authHeader = req.headers.authorization;

    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      throw new UnauthorizedError('Missing or invalid authorization header');
    }

    // Extract token (in real app, verify JWT here)
    const token = authHeader.substring(7);

    if (!token) {
      throw new UnauthorizedError('Missing token');
    }

    // Mock: In real app, decode JWT and load user from DB
    // For demo, create a mock user based on token
    const mockUser: AuthUser = {
      id: 'user-123',
      email: 'user@example.com',
      name: 'Test User',
    };

    // Attach user to request (transforms Request → AuthenticatedRequest)
    (req as any).user = mockUser;

    next();
  } catch (error) {
    next(error);
  }
};

/**
 * Optional auth middleware
 *
 * Similar to requireAuth but doesn't throw if no token is provided
 * Useful for routes that work both with and without authentication
 *
 * Usage:
 * router.get('/posts', optionalAuth, getPostsHandler);
 */
export const optionalAuth = async (req: Request, res: Response, next: NextFunction) => {
  try {
    const authHeader = req.headers.authorization;

    if (authHeader && authHeader.startsWith('Bearer ')) {
      const token = authHeader.substring(7);

      if (token) {
        // Mock user (in real app, decode and load from DB)
        const mockUser: AuthUser = {
          id: 'user-123',
          email: 'user@example.com',
          name: 'Test User',
        };

        (req as any).user = mockUser;
      }
    }

    next();
  } catch (error) {
    // On error, just continue without user (don't block the request)
    next();
  }
};
