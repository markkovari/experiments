import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import cors from 'cors';
import express from 'express';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load proto
const PROTO_PATH = join(__dirname, '../backend/todo.proto');
const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true
});
const todoProto = grpc.loadPackageDefinition(packageDefinition).todo;

// Create gRPC client
const client = new todoProto.TodoService(
  'localhost:50051',
  grpc.credentials.createInsecure()
);

const app = express();
app.use(cors());
app.use(express.json());

// REST endpoints that proxy to gRPC
app.get('/todos', (_req, res) => {
  console.log('\n[PROXY] REST GET /todos');
  console.log('[PROXY] Calling gRPC getTodos() with empty request');
  client.getTodos({}, (err, response) => {
    if (err) {
      console.log('[PROXY] gRPC error:', err.message);
      res.status(500).json({ error: err.message });
      return;
    }
    console.log('[PROXY] gRPC response:', JSON.stringify(response, null, 2));
    console.log('[PROXY] Sending JSON to client');
    res.json(response.todos);
  });
});

app.post('/todos', (req, res) => {
  const { text } = req.body;
  console.log('\n[PROXY] REST POST /todos');
  console.log('[PROXY] JSON body:', JSON.stringify(req.body, null, 2));
  console.log('[PROXY] Calling gRPC addTodo() with:', { text });
  client.addTodo({ text }, (err, response) => {
    if (err) {
      console.log('[PROXY] gRPC error:', err.message);
      res.status(500).json({ error: err.message });
      return;
    }
    console.log('[PROXY] gRPC response:', JSON.stringify(response, null, 2));
    console.log('[PROXY] Sending JSON to client');
    res.status(201).json(response);
  });
});

app.patch('/todos/:id/toggle', (req, res) => {
  const { id } = req.params;
  console.log('\n[PROXY] REST PATCH /todos/:id/toggle');
  console.log('[PROXY] Calling gRPC toggleTodo() with:', { id });
  client.toggleTodo({ id }, (err, response) => {
    if (err) {
      console.log('[PROXY] gRPC error:', err.message);
      res.status(500).json({ error: err.message });
      return;
    }
    console.log('[PROXY] gRPC response:', JSON.stringify(response, null, 2));
    res.json(response);
  });
});

app.put('/todos/:id', (req, res) => {
  const { id } = req.params;
  const { text } = req.body;
  console.log('\n[PROXY] REST PUT /todos/:id');
  console.log('[PROXY] JSON body:', JSON.stringify(req.body, null, 2));
  console.log('[PROXY] Calling gRPC updateTodo() with:', { id, text });
  client.updateTodo({ id, text }, (err, response) => {
    if (err) {
      console.log('[PROXY] gRPC error:', err.message);
      res.status(500).json({ error: err.message });
      return;
    }
    console.log('[PROXY] gRPC response:', JSON.stringify(response, null, 2));
    res.json(response);
  });
});

app.delete('/todos/:id', (req, res) => {
  const { id } = req.params;
  console.log('\n[PROXY] REST DELETE /todos/:id');
  console.log('[PROXY] Calling gRPC deleteTodo() with:', { id });
  client.deleteTodo({ id }, (err, _response) => {
    if (err) {
      console.log('[PROXY] gRPC error:', err.message);
      res.status(500).json({ error: err.message });
      return;
    }
    console.log('[PROXY] gRPC response: { success: true }');
    res.status(204).send();
  });
});

const PORT = 3001;
app.listen(PORT, () => {
  console.log(`\ngRPC proxy running on http://localhost:${PORT}`);
  console.log('Converts REST/JSON → gRPC/Protobuf → REST/JSON');
  console.log('Watch the logs to see the message conversion!\n');
});
