import { randomUUID } from 'node:crypto';
import type Surreal from 'surrealdb';
import { afterAll, beforeAll } from 'vitest';
import { createDbClient } from '../src/db/client.js';

let db: Surreal;

// Each test file gets a unique database name for isolation
const uniqueDbName = `test_${randomUUID().replace(/-/g, '_')}`;

export function getDb(): Surreal {
  return db;
}

export function getContainerUrl(): string {
  // Use HTTP (SurrealDB JS client handles protocol internally)
  return process.env.SURREAL_TEST_URL || 'http://localhost:8000';
}

beforeAll(async () => {
  const url = getContainerUrl();

  db = await createDbClient({
    url,
    namespace: 'test',
    database: uniqueDbName,
    username: 'root',
    password: 'root',
  });
});

afterAll(async () => {
  if (db) {
    await db.close();
  }
});
