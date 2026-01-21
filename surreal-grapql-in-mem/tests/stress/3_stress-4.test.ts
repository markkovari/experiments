import { describe, it, expect, beforeEach } from 'vitest';
import { ApolloServer } from '@apollo/server';
import { typeDefs } from '../../src/schema/index.js';
import { resolvers, type Context } from '../../src/resolvers/index.js';
import { getDb } from '../setup.js';

describe('Stress Tests - Batch 4 (Mixed & Special Cases)', () => {
  let server: ApolloServer<Context>;

  beforeEach(async () => {
    const db = getDb();
    await db.query('DELETE user');
    server = new ApolloServer<Context>({ typeDefs, resolvers });
  });

  it('should handle mixed CRUD operations on 25 users concurrently', async () => {
    const db = getDb();
    const userCount = 25;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Mixed User ${i}`, email: `mixed${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    const operations: Array<Promise<unknown>> = [];

    // Create 12 new users
    for (let i = 0; i < 12; i++) {
      operations.push(
        server.executeOperation(
          {
            query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
            variables: { input: { name: `New Mixed User ${i}`, email: `newmixed${i}@example.com` } },
          },
          { contextValue: { db } }
        )
      );
    }

    // Update first 12 users
    for (let i = 0; i < 12; i++) {
      operations.push(
        server.executeOperation(
          {
            query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id } }`,
            variables: { id: createdUsers[i].id, input: { name: `Updated Mixed User ${i}` } },
          },
          { contextValue: { db } }
        )
      );
    }

    // Delete last 13 users
    for (let i = 12; i < 25; i++) {
      operations.push(
        server.executeOperation(
          { query: `mutation DeleteUser($id: ID!) { deleteUser(id: $id) }`, variables: { id: createdUsers[i].id } },
          { contextValue: { db } }
        )
      );
    }

    await Promise.all(operations);

    const queryResponse = await server.executeOperation(
      { query: `query GetUsers { users { id } }` },
      { contextValue: { db } }
    );

    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ id: string }> };
      expect(data.users).toHaveLength(24); // 12 original + 12 new
    }
  });

  it('should handle 50 users with long names', async () => {
    const db = getDb();
    const userCount = 50;
    const longName = 'A'.repeat(200);

    const createPromises = Array.from({ length: userCount }, (_, i) =>
      server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: `${longName} ${i}`, email: `longname${i}@example.com` } },
        },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(createPromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);
  });

  it('should handle 50 users with special characters in email', async () => {
    const db = getDb();
    const userCount = 50;

    const createPromises = Array.from({ length: userCount }, (_, i) =>
      server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id email } }`,
          variables: { input: { name: `Special User ${i}`, email: `special+tag${i}@sub.example.com` } },
        },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(createPromises);
    results.forEach((response, i) => {
      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        const data = response.body.singleResult.data as { createUser: { email: string } };
        expect(data.createUser.email).toBe(`special+tag${i}@sub.example.com`);
      }
    });
  });

  it('should handle 60 users with unicode names', async () => {
    const db = getDb();
    const userCount = 60;
    const unicodeNames = ['日本語', '中文', '한국어', 'العربية', 'עברית', 'ไทย', 'Ελληνικά', 'Русский'];

    const createPromises = Array.from({ length: userCount }, (_, i) =>
      server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: `${unicodeNames[i % unicodeNames.length]} User ${i}`, email: `unicode${i}@example.com` } },
        },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(createPromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);

    const queryResponse = await server.executeOperation(
      { query: `query GetUsers { users { name } }` },
      { contextValue: { db } }
    );

    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ name: string }> };
      expect(data.users).toHaveLength(userCount);
    }
  });

  it('should handle 40 concurrent update-then-read operations', async () => {
    const db = getDb();
    const userCount = 40;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `UpdateRead User ${i}`, email: `updateread${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    const operations = createdUsers.map(async ({ id }, i) => {
      await server.executeOperation(
        {
          query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id } }`,
          variables: { id, input: { name: `Updated UR ${i}` } },
        },
        { contextValue: { db } }
      );

      return server.executeOperation(
        { query: `query GetUser($id: ID!) { user(id: $id) { name } }`, variables: { id } },
        { contextValue: { db } }
      );
    });

    const results = await Promise.all(operations);
    results.forEach((response, i) => {
      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        const data = response.body.singleResult.data as { user: { name: string } };
        expect(data.user.name).toBe(`Updated UR ${i}`);
      }
    });
  });

  it('should handle 1000 users creation via direct DB and GraphQL query', async () => {
    const db = getDb();
    const userCount = 1000;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Huge User ${i}`, email: `huge${i}@example.com` }));
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

  it('should handle 1500 users creation and retrieval', async () => {
    const db = getDb();
    const userCount = 1500;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Massive User ${i}`, email: `massive${i}@example.com` }));
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

  it('should handle 200 parallel mutations across create/update/delete', async () => {
    const db = getDb();

    // Batch insert 80 users
    const users = Array.from({ length: 80 }, (_, i) => ({ name: `Pre User ${i}`, email: `pre${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    const operations: Array<Promise<unknown>> = [];

    // 80 creates
    for (let i = 0; i < 80; i++) {
      operations.push(
        server.executeOperation(
          {
            query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
            variables: { input: { name: `Parallel Mix ${i}`, email: `parallelmix${i}@example.com` } },
          },
          { contextValue: { db } }
        )
      );
    }

    // 80 updates
    for (let i = 0; i < 80; i++) {
      operations.push(
        server.executeOperation(
          {
            query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id } }`,
            variables: { id: createdUsers[i].id, input: { name: `Updated Parallel ${i}` } },
          },
          { contextValue: { db } }
        )
      );
    }

    // 40 reads
    for (let i = 0; i < 40; i++) {
      operations.push(
        server.executeOperation(
          { query: `query GetUser($id: ID!) { user(id: $id) { id } }`, variables: { id: createdUsers[i].id } },
          { contextValue: { db } }
        )
      );
    }

    const results = await Promise.all(operations);
    const successCount = results.filter((r) => (r as { body: { kind: string } }).body.kind === 'single').length;
    expect(successCount).toBe(200);
  });
});
