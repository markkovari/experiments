import { z } from 'zod';

/**
 * Zod schemas for user validation
 *
 * These schemas provide both:
 * 1. Runtime validation (ensures data matches the schema)
 * 2. Type inference (TypeScript knows the exact shape)
 */

// Schema for creating a new user
export const createUserSchema = z.object({
  email: z.string().email('Invalid email format'),
  name: z.string().min(1, 'Name is required').max(100, 'Name is too long'),
  password: z.string().min(6, 'Password must be at least 6 characters'),
});

// Infer TypeScript type from schema
export type CreateUserBody = z.infer<typeof createUserSchema>;

// Schema for updating a user
export const updateUserSchema = z.object({
  name: z.string().min(1, 'Name is required').max(100, 'Name is too long').optional(),
  email: z.string().email('Invalid email format').optional(),
});

export type UpdateUserBody = z.infer<typeof updateUserSchema>;

// Schema for user ID param
export const userIdParamSchema = z.object({
  id: z.string().uuid('Invalid user ID format'),
});

export type UserIdParams = z.infer<typeof userIdParamSchema>;
