import { describe, it, expect, beforeEach } from 'vitest';
import { ApolloServer } from '@apollo/server';
import { typeDefs } from '../../src/schema/index.js';
import { resolvers, type Context } from '../../src/resolvers/index.js';
import { getDb } from '../setup.js';

describe('Stress Tests - Batch 2 (Read Operations)', () => {
  let server: ApolloServer<Context>;

  beforeEach(async () => {
    const db = getDb();
    await db.query('DELETE user');
    server = new ApolloServer<Context>({ typeDefs, resolvers });
  });

  it('should handle rapid sequential reads of 100 users', async () => {
    const db = getDb();
    const userCount = 100;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Read User ${i}`, email: `read${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    for (const { id } of createdUsers) {
      const response = await server.executeOperation(
        { query: `query GetUser($id: ID!) { user(id: $id) { id name } }`, variables: { id } },
        { contextValue: { db } }
      );
      expect(response.body.kind).toBe('single');
    }
  });

  it('should handle parallel reads of 50 users', async () => {
    const db = getDb();
    const userCount = 50;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Parallel User ${i}`, email: `parallel${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    const queryPromises = createdUsers.map(({ id }) =>
      server.executeOperation(
        { query: `query GetUser($id: ID!) { user(id: $id) { id name } }`, variables: { id } },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(queryPromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);
  });

  it('should handle querying 200 users multiple times', async () => {
    const db = getDb();
    const userCount = 200;
    const queryCount = 5;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `MultiQuery User ${i}`, email: `multiquery${i}@example.com` }));
    await db.query(`INSERT INTO user $users`, { users });

    for (let q = 0; q < queryCount; q++) {
      const response = await server.executeOperation(
        { query: `query GetUsers { users { id name } }` },
        { contextValue: { db } }
      );
      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        const data = response.body.singleResult.data as { users: Array<{ id: string }> };
        expect(data.users).toHaveLength(userCount);
      }
    }
  });

  it('should handle querying non-existent users 100 times', async () => {
    const db = getDb();
    const queryCount = 100;

    const queryPromises = Array.from({ length: queryCount }, (_, i) =>
      server.executeOperation(
        { query: `query GetUser($id: ID!) { user(id: $id) { id } }`, variables: { id: `user:nonexistent${i}` } },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(queryPromises);
    results.forEach((response) => {
      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        const data = response.body.singleResult.data as { user: null };
        expect(data.user).toBeNull();
      }
    });
  });

  it('should handle 150 parallel single-user queries', async () => {
    const db = getDb();

    const [user] = await db.create('user', { name: 'Single Target', email: 'singletarget@example.com' });
    const userId = user.id.toString();

    const queryPromises = Array.from({ length: 150 }, () =>
      server.executeOperation(
        { query: `query GetUser($id: ID!) { user(id: $id) { id name } }`, variables: { id: userId } },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(queryPromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(150);
  });

  it('should handle 100 users with batch queries', async () => {
    const db = getDb();
    const userCount = 100;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Batch User ${i}`, email: `batch${i}@example.com` }));
    await db.query(`INSERT INTO user $users`, { users });

    for (let batch = 0; batch < 3; batch++) {
      const response = await server.executeOperation(
        { query: `query GetUsers { users { id name } }` },
        { contextValue: { db } }
      );
      expect(response.body.kind).toBe('single');
    }
  });

  it('should handle 500 users creation and retrieval', async () => {
    const db = getDb();
    const userCount = 500;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Large User ${i}`, email: `large${i}@example.com` }));
    await db.query(`INSERT INTO user $users`, { users });

    const queryResponse = await server.executeOperation(
      { query: `query GetUsers { users { id name } }` },
      { contextValue: { db } }
    );

    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ id: string }> };
      expect(data.users).toHaveLength(userCount);
    }
  });
});
