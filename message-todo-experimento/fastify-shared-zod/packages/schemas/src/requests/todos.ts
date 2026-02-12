import { z } from 'zod';

/**
 * Request schemas for Todo endpoints
 */

export const addTodoRequestSchema = z.object({
  text: z.string().min(1, 'Text is required'),
});

export const todoIdParamsSchema = z.object({
  id: z.string(),
});

export const toggleTodoRequestSchema = z.object({
  id: z.string(),
});

export const updateTodoRequestSchema = z.object({
  id: z.string(),
  text: z.string().min(1, 'Text is required'),
});

export const deleteTodoRequestSchema = z.object({
  id: z.string(),
});

export type AddTodoRequest = z.infer<typeof addTodoRequestSchema>;
export type TodoIdParams = z.infer<typeof todoIdParamsSchema>;
export type ToggleTodoRequest = z.infer<typeof toggleTodoRequestSchema>;
export type UpdateTodoRequest = z.infer<typeof updateTodoRequestSchema>;
export type DeleteTodoRequest = z.infer<typeof deleteTodoRequestSchema>;
