import cors from "@fastify/cors";
import Database from "better-sqlite3";
import Fastify from "fastify";
import {
	type ZodTypeProvider,
	serializerCompiler,
	validatorCompiler,
} from "fastify-type-provider-zod";
import {
	TodoEndpoints,
	addTodoRequestSchema,
	todoIdParamsSchema,
	todoListResponseSchema,
	todoResponseSchema,
	type Todo,
} from "schemas";

const db = new Database("todos.db");

// Initialize database
db.exec(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text TEXT NOT NULL,
    isDone INTEGER DEFAULT 0
  )
`);

const app = Fastify().withTypeProvider<ZodTypeProvider>();

// Set validator and serializer for Zod
app.setValidatorCompiler(validatorCompiler);
app.setSerializerCompiler(serializerCompiler);

// Register CORS plugin
await app.register(cors, { origin: true });

// List all todos
app.get(TodoEndpoints.list.path, {
	schema: {
		response: {
			200: todoListResponseSchema,
		},
	},
	handler: async (_request, reply) => {
		const todos = db
			.prepare("SELECT id, text, isDone FROM todos")
			.all() as Array<{
			id: number;
			text: string;
			isDone: number;
		}>;

		const response = todos.map((t) => ({
			id: t.id.toString(),
			text: t.text,
			isDone: Boolean(t.isDone),
		}));

		return reply.send(response);
	},
});

// Add todo
app.post(TodoEndpoints.add.path, {
	schema: {
		body: addTodoRequestSchema,
		response: {
			201: todoResponseSchema,
		},
	},
	handler: async ({ body: { text } }, reply) => {
		const result = db.prepare("INSERT INTO todos (text) VALUES (?)").run(text);
		const response: Todo = {
			id: result.lastInsertRowid.toString(),
			text,
			isDone: false,
		};

		return reply.code(201).send(response);
	},
});

// Toggle done
app.patch(TodoEndpoints.toggle.path, {
	schema: {
		params: todoIdParamsSchema,
		response: {
			200: todoResponseSchema,
		},
	},
	handler: async ({ params: { id } }, reply) => {
		db.prepare("UPDATE todos SET isDone = NOT isDone WHERE id = ?").run(id);
		const todo = db
			.prepare("SELECT id, text, isDone FROM todos WHERE id = ?")
			.get(id) as {
			id: number;
			text: string;
			isDone: number;
		};

		const response: Todo = {
			id: todo.id.toString(),
			text: todo.text,
			isDone: Boolean(todo.isDone),
		};

		return reply.send(response);
	},
});

// Update text
app.put(TodoEndpoints.update.path, {
	schema: {
		params: todoIdParamsSchema,
		body: addTodoRequestSchema,
		response: {
			200: todoResponseSchema,
		},
	},
	handler: async ({ params: { id }, body: { text } }, reply) => {
		db.prepare("UPDATE todos SET text = ? WHERE id = ?").run(text, id);
		const todo = db
			.prepare("SELECT id, text, isDone FROM todos WHERE id = ?")
			.get(id) as {
			id: number;
			text: string;
			isDone: number;
		};

		const response: Todo = {
			id: todo.id.toString(),
			text: todo.text,
			isDone: Boolean(todo.isDone),
		};

		return reply.send(response);
	},
});

// Delete todo
app.delete(TodoEndpoints.delete.path, {
	schema: {
		params: todoIdParamsSchema,
	},
	handler: async ({ params: { id } }, reply) => {
		db.prepare("DELETE FROM todos WHERE id = ?").run(id);
		return reply.code(204).send();
	},
});

const PORT = 3004;
await app.listen({ port: PORT });
console.log(
	`\nFastify + Shared Zod Schemas backend running on http://localhost:${PORT}`,
);
console.log("Using Fastify type provider for automatic Zod validation!\n");
