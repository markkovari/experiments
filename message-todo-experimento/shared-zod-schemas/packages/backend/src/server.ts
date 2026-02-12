import Database from 'better-sqlite3';
import cors from 'cors';
import express, { type RequestHandler } from 'express';
import {
  TodoEndpoints,
  addTodoRequestSchema,
  todoListResponseSchema,
  todoResponseSchema,
  type Todo,
} from 'schemas';

const db = new Database('todos.db');

// Initialize database
db.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    isDone INTEGER DEFAULT 0
  )
`);

const app = express();
app.use(cors());
app.use(express.json());

// Helper to register routes using endpoint definitions
function registerRoute<T extends { method: string; path: string }>(
  endpoint: T,
  handler: RequestHandler
): void {
  const method = endpoint.method.toLowerCase() as keyof typeof app;
  (app[method] as any)(endpoint.path, handler);
}

// List all todos
registerRoute(TodoEndpoints.list, (_req, res) => {
  const todos = db.prepare('SELECT id, text, isDone FROM todos').all() as Array<{
    id: number;
    text: string;
    isDone: number;
  }>;

  const response = todos.map((t) => ({
    id: t.id.toString(),
    text: t.text,
    isDone: Boolean(t.isDone),
  }));

  // Validate response with shared schema
  const validated = todoListResponseSchema.parse(response);
  res.json(validated);
});

// Add todo
registerRoute(TodoEndpoints.add, (req, res) => {
  try {
    // Validate request with shared schema
    const { text } = addTodoRequestSchema.parse(req.body);

    const result = db.prepare('INSERT INTO todos (text) VALUES (?)').run(text);
    const response: Todo = {
      id: result.lastInsertRowid.toString(),
      text,
      isDone: false,
    };

    // Validate response with shared schema
    const validated = todoResponseSchema.parse(response);
    res.status(201).json(validated);
  } catch (_error) {
    res.status(400).json({ error: 'Invalid request data' });
  }
});

// Toggle done
registerRoute(TodoEndpoints.toggle, (req, res) => {
  try {
    const { id } = req.params;

    db.prepare('UPDATE todos SET isDone = NOT isDone WHERE id = ?').run(id);
    const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(id) as {
      id: number;
      text: string;
      isDone: number;
    };

    const response: Todo = {
      id: todo.id.toString(),
      text: todo.text,
      isDone: Boolean(todo.isDone),
    };

    // Validate response with shared schema
    const validated = todoResponseSchema.parse(response);
    res.json(validated);
  } catch (_error) {
    res.status(400).json({ error: 'Invalid request' });
  }
});

// Update text
registerRoute(TodoEndpoints.update, (req, res) => {
  try {
    const { id } = req.params;
    // Validate request with shared schema
    const { text } = addTodoRequestSchema.parse(req.body);

    db.prepare('UPDATE todos SET text = ? WHERE id = ?').run(text, id);
    const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(id) as {
      id: number;
      text: string;
      isDone: number;
    };

    const response: Todo = {
      id: todo.id.toString(),
      text: todo.text,
      isDone: Boolean(todo.isDone),
    };

    // Validate response with shared schema
    const validated = todoResponseSchema.parse(response);
    res.json(validated);
  } catch (_error) {
    res.status(400).json({ error: 'Invalid request data' });
  }
});

// Delete todo
registerRoute(TodoEndpoints.delete, (req, res) => {
  const { id } = req.params;
  db.prepare('DELETE FROM todos WHERE id = ?').run(id);
  res.status(204).send();
});

const PORT = 3003;
app.listen(PORT, () => {
  console.log(`\nShared Zod Schemas backend running on http://localhost:${PORT}`);
  console.log('Using shared schemas for validation on backend and frontend!\n');
});
