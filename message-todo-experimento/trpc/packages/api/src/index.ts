import { initTRPC } from '@trpc/server';
import { z } from 'zod';

// Initialize tRPC
const t = initTRPC.create();

// Export types and helpers
export const router = t.router;
export const publicProcedure = t.procedure;

// Define the Todo schema
export const todoSchema = z.object({
  id: z.string(),
  text: z.string(),
  isDone: z.boolean(),
});

export type Todo = z.infer<typeof todoSchema>;

// Define the app router - this is just the type/shape
// The implementation will be in the backend
export const appRouter = router({
  todos: router({
    list: publicProcedure.query<Todo[]>(() => {
      throw new Error('Not implemented');
    }),
    add: publicProcedure
      .input(z.object({ text: z.string() }))
      .mutation<Todo>(() => {
        throw new Error('Not implemented');
      }),
    toggle: publicProcedure
      .input(z.object({ id: z.string() }))
      .mutation<Todo>(() => {
        throw new Error('Not implemented');
      }),
    update: publicProcedure
      .input(z.object({ id: z.string(), text: z.string() }))
      .mutation<Todo>(() => {
        throw new Error('Not implemented');
      }),
    delete: publicProcedure
      .input(z.object({ id: z.string() }))
      .mutation<void>(() => {
        throw new Error('Not implemented');
      }),
  }),
});

// Export the app router type - this is what gives us type safety
export type AppRouter = typeof appRouter;
