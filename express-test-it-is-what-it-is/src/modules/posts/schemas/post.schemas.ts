import { z } from 'zod';

/**
 * Zod schemas for post validation
 */

// Schema for creating a new post
export const createPostSchema = z.object({
  title: z.string().min(1, 'Title is required').max(200, 'Title is too long'),
  content: z.string().min(1, 'Content is required'),
  authorId: z.string().uuid('Invalid author ID format'),
  published: z.boolean().optional().default(false),
});

export type CreatePostBody = z.infer<typeof createPostSchema>;

// Schema for updating a post
export const updatePostSchema = z.object({
  title: z.string().min(1, 'Title is required').max(200, 'Title is too long').optional(),
  content: z.string().min(1, 'Content is required').optional(),
  published: z.boolean().optional(),
});

export type UpdatePostBody = z.infer<typeof updatePostSchema>;

// Schema for post ID param
export const postIdParamSchema = z.object({
  id: z.string().uuid('Invalid post ID format'),
});

export type PostIdParams = z.infer<typeof postIdParamSchema>;

// Schema for author ID param
export const authorIdParamSchema = z.object({
  authorId: z.string().uuid('Invalid author ID format'),
});

export type AuthorIdParams = z.infer<typeof authorIdParamSchema>;
