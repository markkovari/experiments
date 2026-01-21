import { describe, it, expect, beforeEach } from 'vitest';
import { ApolloServer } from '@apollo/server';
import { typeDefs } from '../../src/schema/index.js';
import { resolvers, type Context } from '../../src/resolvers/index.js';
import { getDb } from '../setup.js';

describe('Stress Tests - Batch 1 (Bulk Operations)', () => {
  let server: ApolloServer<Context>;

  beforeEach(async () => {
    const db = getDb();
    await db.query('DELETE user');
    server = new ApolloServer<Context>({ typeDefs, resolvers });
  });

  it('should handle bulk creation of 50 users', async () => {
    const db = getDb();
    const userCount = 50;

    const createPromises = Array.from({ length: userCount }, (_, i) =>
      server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id name email } }`,
          variables: { input: { name: `Stress User ${i}`, email: `stress${i}@example.com` } },
        },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(createPromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);
  });

  it('should handle bulk creation of 200 users via direct DB', async () => {
    const db = getDb();
    const userCount = 200;

    // Batch insert using SurrealQL
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Bulk User ${i}`, email: `bulk${i}@example.com` }));
    await db.query(`INSERT INTO user $users`, { users });

    const queryResponse = await server.executeOperation(
      { query: `query GetUsers { users { id } }` },
      { contextValue: { db } }
    );

    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ id: string }> };
      expect(data.users).toHaveLength(userCount);
    }
  });

  it('should handle bulk updates of 50 users', async () => {
    const db = getDb();
    const userCount = 50;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Update User ${i}`, email: `update${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    const updatePromises = createdUsers.map(({ id }, i) =>
      server.executeOperation(
        {
          query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id name } }`,
          variables: { id, input: { name: `Updated User ${i}` } },
        },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(updatePromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);
  });

  it('should handle bulk deletes of 50 users', async () => {
    const db = getDb();
    const userCount = 50;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Delete User ${i}`, email: `delete${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    const deletePromises = createdUsers.map(({ id }) =>
      server.executeOperation(
        { query: `mutation DeleteUser($id: ID!) { deleteUser(id: $id) }`, variables: { id } },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(deletePromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);

    const queryResponse = await server.executeOperation({ query: `query GetUsers { users { id } }` }, { contextValue: { db } });
    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ id: string }> };
      expect(data.users).toHaveLength(0);
    }
  });

  it('should handle 100 parallel create mutations', async () => {
    const db = getDb();
    const userCount = 100;

    const createPromises = Array.from({ length: userCount }, (_, i) =>
      server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: `Parallel Create ${i}`, email: `parallelcreate${i}@example.com` } },
        },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(createPromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);
  });

  it('should handle 100 users with only name updates', async () => {
    const db = getDb();
    const userCount = 100;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `NameOnly User ${i}`, email: `nameonly${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    const updatePromises = createdUsers.map(({ id }, i) =>
      server.executeOperation(
        {
          query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id name } }`,
          variables: { id, input: { name: `New Name ${i}` } },
        },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(updatePromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);
  });

  it('should handle 100 users with only email updates', async () => {
    const db = getDb();
    const userCount = 100;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `EmailOnly User ${i}`, email: `emailonly${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    const updatePromises = createdUsers.map(({ id }, i) =>
      server.executeOperation(
        {
          query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id email } }`,
          variables: { id, input: { email: `newemail${i}@example.com` } },
        },
        { contextValue: { db } }
      )
    );

    const results = await Promise.all(updatePromises);
    const successCount = results.filter((r) => r.body.kind === 'single' && !r.body.singleResult.errors).length;
    expect(successCount).toBe(userCount);
  });
});
