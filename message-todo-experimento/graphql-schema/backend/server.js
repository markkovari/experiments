import { ApolloServer } from '@apollo/server';
import { expressMiddleware } from '@apollo/server/express4';
import Database from 'better-sqlite3';
import cors from 'cors';
import express from 'express';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const db = new Database('todos.db');

// Initialize database
db.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    isDone INTEGER DEFAULT 0
  )
`);

// Load schema from file
const typeDefs = readFileSync(join(__dirname, 'schema.graphql'), 'utf-8');

// Resolvers
const resolvers = {
  Query: {
    todos: () => {
      const todos = db.prepare('SELECT id, text, isDone FROM todos').all();
      return todos.map(t => ({ ...t, isDone: Boolean(t.isDone) }));
    }
  },
  Mutation: {
    addTodo: (_parent, { text }) => {
      const result = db.prepare('INSERT INTO todos (text) VALUES (?)').run(text);
      return { id: result.lastInsertRowid.toString(), text, isDone: false };
    },
    toggleTodo: (_parent, { id }) => {
      db.prepare('UPDATE todos SET isDone = NOT isDone WHERE id = ?').run(id);
      const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(id);
      return { ...todo, id: todo.id.toString(), isDone: Boolean(todo.isDone) };
    },
    updateTodo: (_parent, { id, text }) => {
      db.prepare('UPDATE todos SET text = ? WHERE id = ?').run(text, id);
      const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(id);
      return { ...todo, id: todo.id.toString(), isDone: Boolean(todo.isDone) };
    },
    deleteTodo: (_parent, { id }) => {
      db.prepare('DELETE FROM todos WHERE id = ?').run(id);
      return true;
    }
  }
};

const server = new ApolloServer({
  typeDefs,
  resolvers
});

await server.start();

const app = express();
app.use(cors());
app.use(express.json());

app.use('/graphql', expressMiddleware(server));

const PORT = 4000;
app.listen(PORT, () => {
  console.log(`GraphQL server running on http://localhost:${PORT}/graphql`);
});
