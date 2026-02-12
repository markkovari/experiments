import { z } from 'zod';

/**
 * Core Todo domain model
 */
export const todoSchema = z.object({
  id: z.string(),
  text: z.string(),
  isDone: z.boolean(),
});

export type Todo = z.infer<typeof todoSchema>;
