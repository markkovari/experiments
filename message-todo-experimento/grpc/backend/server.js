import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import Database from 'better-sqlite3';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Enable gRPC logging to see more details (set via environment variable)
// To see raw wire format: GRPC_TRACE=all GRPC_VERBOSITY=DEBUG node server.js
// Note: gRPC uses binary Protocol Buffers on the wire. The logs below show
// the deserialized JSON representation. To see actual binary data, use:
// - Wireshark with gRPC dissector
// - tcpdump + protoc --decode
// - gRPC interceptors to log raw bytes

const db = new Database('todos.db');

// Initialize database
db.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    isDone INTEGER DEFAULT 0
  )
`);

// Load proto
const PROTO_PATH = join(__dirname, 'todo.proto');
const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true
});
const todoProto = grpc.loadPackageDefinition(packageDefinition).todo;

// Implement service methods
const getTodos = (_call, callback) => {
  console.log('\n[BACKEND] gRPC getTodos() called');
  console.log('[BACKEND] Request: (empty)');
  const todos = db.prepare('SELECT id, text, isDone FROM todos').all();
  const response = {
    todos: todos.map(t => ({
      id: t.id.toString(),
      text: t.text,
      isDone: Boolean(t.isDone)
    }))
  };
  console.log('[BACKEND] Response (serialized to protobuf):', JSON.stringify(response, null, 2));
  callback(null, response);
};

const addTodo = (call, callback) => {
  console.log('\n[BACKEND] gRPC addTodo() called');
  console.log('[BACKEND] Request (deserialized from protobuf):', JSON.stringify(call.request, null, 2));
  const { text } = call.request;
  const result = db.prepare('INSERT INTO todos (text) VALUES (?)').run(text);
  const response = {
    id: result.lastInsertRowid.toString(),
    text,
    isDone: false
  };
  console.log('[BACKEND] Response (serialized to protobuf):', JSON.stringify(response, null, 2));
  callback(null, response);
};

const toggleTodo = (call, callback) => {
  console.log('\n[BACKEND] gRPC toggleTodo() called');
  console.log('[BACKEND] Request (deserialized from protobuf):', JSON.stringify(call.request, null, 2));
  const { id } = call.request;
  db.prepare('UPDATE todos SET isDone = NOT isDone WHERE id = ?').run(id);
  const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(id);
  const response = {
    id: todo.id.toString(),
    text: todo.text,
    isDone: Boolean(todo.isDone)
  };
  console.log('[BACKEND] Response (serialized to protobuf):', JSON.stringify(response, null, 2));
  callback(null, response);
};

const updateTodo = (call, callback) => {
  console.log('\n[BACKEND] gRPC updateTodo() called');
  console.log('[BACKEND] Request (deserialized from protobuf):', JSON.stringify(call.request, null, 2));
  const { id, text } = call.request;
  db.prepare('UPDATE todos SET text = ? WHERE id = ?').run(text, id);
  const todo = db.prepare('SELECT id, text, isDone FROM todos WHERE id = ?').get(id);
  const response = {
    id: todo.id.toString(),
    text: todo.text,
    isDone: Boolean(todo.isDone)
  };
  console.log('[BACKEND] Response (serialized to protobuf):', JSON.stringify(response, null, 2));
  callback(null, response);
};

const deleteTodo = (call, callback) => {
  console.log('\n[BACKEND] gRPC deleteTodo() called');
  console.log('[BACKEND] Request (deserialized from protobuf):', JSON.stringify(call.request, null, 2));
  const { id } = call.request;
  db.prepare('DELETE FROM todos WHERE id = ?').run(id);
  const response = { success: true };
  console.log('[BACKEND] Response (serialized to protobuf):', JSON.stringify(response, null, 2));
  callback(null, response);
};

// Create and start server
const server = new grpc.Server();
server.addService(todoProto.TodoService.service, {
  getTodos,
  addTodo,
  toggleTodo,
  updateTodo,
  deleteTodo
});

const PORT = '0.0.0.0:50051';
server.bindAsync(PORT, grpc.ServerCredentials.createInsecure(), (err, port) => {
  if (err) {
    console.error('Failed to bind server:', err);
    return;
  }
  console.log(`\ngRPC server running on port ${port}`);
  console.log('Messages are transmitted as binary Protocol Buffers (efficient, compact)');
  console.log('Logs show deserialized JSON for readability');
  console.log('To see raw protobuf: GRPC_TRACE=all GRPC_VERBOSITY=DEBUG node server.js\n');
});
