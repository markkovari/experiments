import { z } from 'zod';
import { todoSchema } from '../models/todo';

/**
 * Response schemas for Todo endpoints
 */

export const todoResponseSchema = todoSchema;

export const todoListResponseSchema = z.array(todoSchema);

export const deleteResponseSchema = z.object({
  success: z.boolean(),
});

export type TodoResponse = z.infer<typeof todoResponseSchema>;
export type TodoListResponse = z.infer<typeof todoListResponseSchema>;
export type DeleteResponse = z.infer<typeof deleteResponseSchema>;
