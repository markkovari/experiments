import * as trpcExpress from '@trpc/server/adapters/express';
import { publicProcedure, router, type Todo } from 'api';
import Database from 'better-sqlite3';
import cors from 'cors';
import express from 'express';
import { z } from 'zod';

const db = new Database('todos.db');

// Initialize database
db.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    isDone INTEGER DEFAULT 0
  )
`);

// Implement the actual router
const appRouter = router({
  todos: router({
    list: publicProcedure.query((): Todo[] => {
      const todos = db.prepare('SELECT id, text, isDone FROM todos').all() as Array<{
        id: number;
        text: string;
        isDone: number;
      }>;
      return todos.map((t) => ({
        id: t.id.toString(),
        text: t.text,
        isDone: Boolean(t.isDone),
      }));
    }),

    add: publicProcedure
      .input(z.object({ text: z.string() }))
      .mutation(({ input }): Todo => {
        const result = db.prepare('INSERT INTO todos (text) VALUES (?)').run(input.text);
        return {
          id: result.lastInsertRowid.toString(),
          text: input.text,
          isDone: false,
        };
      }),

    toggle: publicProcedure
      .input(z.object({ id: z.string() }))
      .mutation(({ input }): Todo => {
        db.prepare('UPDATE todos SET isDone = NOT isDone WHERE id = ?').run(input.id);
        const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(input.id) as {
          id: number;
          text: string;
          isDone: number;
        };
        return {
          id: todo.id.toString(),
          text: todo.text,
          isDone: Boolean(todo.isDone),
        };
      }),

    update: publicProcedure
      .input(z.object({ id: z.string(), text: z.string() }))
      .mutation(({ input }): Todo => {
        db.prepare('UPDATE todos SET text = ? WHERE id = ?').run(input.text, input.id);
        const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(input.id) as {
          id: number;
          text: string;
          isDone: number;
        };
        return {
          id: todo.id.toString(),
          text: todo.text,
          isDone: Boolean(todo.isDone),
        };
      }),

    delete: publicProcedure
      .input(z.object({ id: z.string() }))
      .mutation(({ input }): void => {
        db.prepare('DELETE FROM todos WHERE id = ?').run(input.id);
      }),
  }),
});

export type AppRouter = typeof appRouter;

// Create Express app
const app = express();
app.use(cors());

// Add tRPC middleware
app.use(
  '/trpc',
  trpcExpress.createExpressMiddleware({
    router: appRouter,
  })
);

const PORT = 3002;
app.listen(PORT, () => {
  console.log(`\ntRPC server running on http://localhost:${PORT}`);
  console.log('Full type safety from backend to frontend!');
  console.log('Frontend gets automatic TypeScript types from the backend\n');
});
