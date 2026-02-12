import { z } from 'zod';

/**
 * Request to add a new todo
 */
export const addTodoRequestSchema = z.object({
  text: z.string().min(1, 'Text is required'),
});

export type AddTodoRequest = z.infer<typeof addTodoRequestSchema>;

/**
 * Request to toggle todo completion status
 */
export const toggleTodoRequestSchema = z.object({
  id: z.string(),
});

export type ToggleTodoRequest = z.infer<typeof toggleTodoRequestSchema>;

/**
 * Request to update todo text
 */
export const updateTodoRequestSchema = z.object({
  id: z.string(),
  text: z.string().min(1, 'Text is required'),
});

export type UpdateTodoRequest = z.infer<typeof updateTodoRequestSchema>;

/**
 * Request to delete a todo
 */
export const deleteTodoRequestSchema = z.object({
  id: z.string(),
});

export type DeleteTodoRequest = z.infer<typeof deleteTodoRequestSchema>;
