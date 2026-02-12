import { z } from 'zod';

/**
 * Todo domain model
 * Represents a todo item with id, text, and completion status
 */
export const todoSchema = z.object({
  id: z.string(),
  text: z.string(),
  isDone: z.boolean(),
});

export type Todo = z.infer<typeof todoSchema>;
