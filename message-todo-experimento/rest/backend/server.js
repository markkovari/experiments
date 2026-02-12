import Database from 'better-sqlite3';
import cors from 'cors';
import express from 'express';

const app = express();
const db = new Database('todos.db');

// Initialize database
db.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    isDone INTEGER DEFAULT 0
  )
`);

app.use(cors());
app.use(express.json());

// List all todos
app.get('/todos', (_req, res) => {
  const todos = db.prepare('SELECT id, text, isDone FROM todos').all();
  res.json(todos.map(t => ({ ...t, isDone: Boolean(t.isDone) })));
});

// Add todo
app.post('/todos', (req, res) => {
  const { text } = req.body;
  const result = db.prepare('INSERT INTO todos (text) VALUES (?)').run(text);
  res.status(201).json({ id: result.lastInsertRowid, text, isDone: false });
});

// Toggle done
app.patch('/todos/:id/toggle', (req, res) => {
  const { id } = req.params;
  db.prepare('UPDATE todos SET isDone = NOT isDone WHERE id = ?').run(id);
  const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(id);
  res.json({ ...todo, isDone: Boolean(todo.isDone) });
});

// Update text
app.put('/todos/:id', (req, res) => {
  const { id } = req.params;
  const { text } = req.body;
  db.prepare('UPDATE todos SET text = ? WHERE id = ?').run(text, id);
  const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(id);
  res.json({ ...todo, isDone: Boolean(todo.isDone) });
});

// Delete todo
app.delete('/todos/:id', (req, res) => {
  const { id } = req.params;
  db.prepare('DELETE FROM todos WHERE id = ?').run(id);
  res.status(204).send();
});

const PORT = 3000;
app.listen(PORT, () => {
  console.log(`REST API running on http://localhost:${PORT}`);
});
