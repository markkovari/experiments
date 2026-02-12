import { z } from 'zod';
import { todoSchema } from '../models/todo';

/**
 * Response for a single todo
 */
export const todoResponseSchema = todoSchema;

export type TodoResponse = z.infer<typeof todoResponseSchema>;

/**
 * Response for list of todos
 */
export const todoListResponseSchema = z.array(todoSchema);

export type TodoListResponse = z.infer<typeof todoListResponseSchema>;

/**
 * Response for delete operation
 */
export const deleteResponseSchema = z.object({
  success: z.boolean(),
});

export type DeleteResponse = z.infer<typeof deleteResponseSchema>;
