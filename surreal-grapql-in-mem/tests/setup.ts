import { GenericContainer, type StartedTestContainer, Wait } from 'testcontainers';
import type Surreal from 'surrealdb';
import { beforeAll, afterAll } from 'vitest';
import { createDbClient } from '../src/db/client.js';

let container: StartedTestContainer;
let db: Surreal;

export function getDb(): Surreal {
  return db;
}

export function getContainerUrl(): string {
  const host = container.getHost();
  const port = container.getMappedPort(8000);
  return `http://${host}:${port}`;
}

beforeAll(async () => {
  container = await new GenericContainer('surrealdb/surrealdb:latest')
    .withExposedPorts(8000)
    .withCommand(['start', '--bind', '0.0.0.0:8000', '--user', 'root', '--pass', 'root', 'memory'])
    .withWaitStrategy(Wait.forLogMessage(/Started web server/))
    .start();

  const url = getContainerUrl();

  db = await createDbClient({
    url,
    namespace: 'test',
    database: 'test',
    username: 'root',
    password: 'root',
  });
});

afterAll(async () => {
  if (db) {
    await db.close();
  }
  if (container) {
    await container.stop();
  }
});
