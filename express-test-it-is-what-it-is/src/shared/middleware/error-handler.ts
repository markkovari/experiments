import { Request, Response, NextFunction } from 'express';
import { ValidationError } from './validation';

/**
 * Custom application errors
 */
export class AppError extends Error {
  constructor(
    public statusCode: number,
    message: string,
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export class NotFoundError extends AppError {
  constructor(resource: string) {
    super(404, `${resource} not found`);
    this.name = 'NotFoundError';
  }
}

export class ConflictError extends AppError {
  constructor(message: string) {
    super(409, message);
    this.name = 'ConflictError';
  }
}

export class UnauthorizedError extends AppError {
  constructor(message: string = 'Unauthorized') {
    super(401, message);
    this.name = 'UnauthorizedError';
  }
}

/**
 * Global error handler middleware
 *
 * Catches all errors and formats them into consistent JSON responses
 * Handles ValidationError, AppError, and generic errors differently
 *
 * Usage: Add as the last middleware in app.ts:
 * app.use(errorHandler);
 */
export const errorHandler = (err: Error, req: Request, res: Response, next: NextFunction) => {
  // Log error for debugging
  console.error('Error:', err);

  // Validation errors (400)
  if (err instanceof ValidationError) {
    return res.status(400).json({
      error: 'Validation failed',
      details: err.errors,
    });
  }

  // Application errors (custom status codes)
  if (err instanceof AppError) {
    return res.status(err.statusCode).json({
      error: err.message,
    });
  }

  // Generic errors (500)
  res.status(500).json({
    error: 'Internal server error',
  });
};
