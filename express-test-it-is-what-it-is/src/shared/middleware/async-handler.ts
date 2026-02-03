import { Request, Response, NextFunction } from 'express';

/**
 * Async handler wrapper - eliminates try-catch repetition
 *
 * Wraps async route handlers and automatically catches errors,
 * passing them to Express error handling middleware
 *
 * Usage:
 * router.get('/users', asyncHandler(async (req, res) => {
 *   const users = await userService.getAll();
 *   res.json(users);
 * }));
 */
export const asyncHandler = (
  fn: (req: Request, res: Response, next: NextFunction) => Promise<any>,
) => {
  return (req: Request, res: Response, next: NextFunction) => {
    Promise.resolve(fn(req, res, next)).catch(next);
  };
};
