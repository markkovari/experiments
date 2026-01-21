import { describe, it, expect, beforeEach } from 'vitest';
import { ApolloServer } from '@apollo/server';
import { typeDefs } from '../../src/schema/index.js';
import { resolvers, type Context } from '../../src/resolvers/index.js';
import { getDb } from '../setup.js';

describe('Stress Tests - Batch 3 (Sequential & Cycle Operations)', () => {
  let server: ApolloServer<Context>;

  beforeEach(async () => {
    const db = getDb();
    await db.query('DELETE user');
    server = new ApolloServer<Context>({ typeDefs, resolvers });
  });

  it('should handle rapid create-read cycles for 50 users', async () => {
    const db = getDb();
    const userCount = 50;

    for (let i = 0; i < userCount; i++) {
      const createResponse = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id name } }`,
          variables: { input: { name: `Cycle User ${i}`, email: `cycle${i}@example.com` } },
        },
        { contextValue: { db } }
      );

      expect(createResponse.body.kind).toBe('single');
      if (createResponse.body.kind === 'single') {
        const data = createResponse.body.singleResult.data as { createUser: { id: string } };
        const userId = data.createUser.id;

        const readResponse = await server.executeOperation(
          { query: `query GetUser($id: ID!) { user(id: $id) { id name } }`, variables: { id: userId } },
          { contextValue: { db } }
        );

        expect(readResponse.body.kind).toBe('single');
        if (readResponse.body.kind === 'single') {
          const readData = readResponse.body.singleResult.data as { user: { name: string } };
          expect(readData.user.name).toBe(`Cycle User ${i}`);
        }
      }
    }
  });

  it('should handle 100 sequential mutations', async () => {
    const db = getDb();
    const mutationCount = 100;

    for (let i = 0; i < mutationCount; i++) {
      const response = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: `Sequential ${i}`, email: `sequential${i}@example.com` } },
        },
        { contextValue: { db } }
      );
      expect(response.body.kind).toBe('single');
    }
  });

  it('should handle create-update-delete cycle for 30 users', async () => {
    const db = getDb();
    const userCount = 30;

    for (let i = 0; i < userCount; i++) {
      const createResponse = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: `CUD User ${i}`, email: `cud${i}@example.com` } },
        },
        { contextValue: { db } }
      );

      expect(createResponse.body.kind).toBe('single');
      if (createResponse.body.kind === 'single') {
        const createData = createResponse.body.singleResult.data as { createUser: { id: string } };
        const userId = createData.createUser.id;

        await server.executeOperation(
          {
            query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id } }`,
            variables: { id: userId, input: { name: `Updated CUD ${i}` } },
          },
          { contextValue: { db } }
        );

        const deleteResponse = await server.executeOperation(
          { query: `mutation DeleteUser($id: ID!) { deleteUser(id: $id) }`, variables: { id: userId } },
          { contextValue: { db } }
        );

        expect(deleteResponse.body.kind).toBe('single');
        if (deleteResponse.body.kind === 'single') {
          const deleteData = deleteResponse.body.singleResult.data as { deleteUser: boolean };
          expect(deleteData.deleteUser).toBe(true);
        }
      }
    }

    const queryResponse = await server.executeOperation(
      { query: `query GetUsers { users { id } }` },
      { contextValue: { db } }
    );

    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ id: string }> };
      expect(data.users).toHaveLength(0);
    }
  });

  it('should handle alternating create-query pattern for 40 iterations', async () => {
    const db = getDb();
    const iterations = 40;

    for (let i = 0; i < iterations; i++) {
      await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: `Alternate User ${i}`, email: `alternate${i}@example.com` } },
        },
        { contextValue: { db } }
      );

      const queryResponse = await server.executeOperation(
        { query: `query GetUsers { users { id } }` },
        { contextValue: { db } }
      );

      expect(queryResponse.body.kind).toBe('single');
      if (queryResponse.body.kind === 'single') {
        const data = queryResponse.body.singleResult.data as { users: Array<{ id: string }> };
        expect(data.users).toHaveLength(i + 1);
      }
    }
  });

  it('should handle interleaved create and delete of 40 users', async () => {
    const db = getDb();
    const userCount = 40;
    const createdIds: string[] = [];

    for (let i = 0; i < userCount; i++) {
      const createResponse = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: `Interleave User ${i}`, email: `interleave${i}@example.com` } },
        },
        { contextValue: { db } }
      );

      if (createResponse.body.kind === 'single') {
        const data = createResponse.body.singleResult.data as { createUser: { id: string } };
        createdIds.push(data.createUser.id);
      }

      if (i % 2 === 1 && createdIds.length >= 2) {
        const idToDelete = createdIds[createdIds.length - 2];
        await server.executeOperation(
          { query: `mutation DeleteUser($id: ID!) { deleteUser(id: $id) }`, variables: { id: idToDelete } },
          { contextValue: { db } }
        );
      }
    }

    const queryResponse = await server.executeOperation(
      { query: `query GetUsers { users { id } }` },
      { contextValue: { db } }
    );

    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ id: string }> };
      expect(data.users.length).toBeGreaterThanOrEqual(20);
    }
  });

  it('should handle 30 users with rapid successive updates', async () => {
    const db = getDb();
    const userCount = 30;
    const updatesPerUser = 5;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Rapid User ${i}`, email: `rapid${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    for (const { id } of createdUsers) {
      for (let u = 0; u < updatesPerUser; u++) {
        await server.executeOperation(
          {
            query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id } }`,
            variables: { id, input: { name: `Rapid Update ${u}` } },
          },
          { contextValue: { db } }
        );
      }
    }

    const queryResponse = await server.executeOperation(
      { query: `query GetUsers { users { name } }` },
      { contextValue: { db } }
    );

    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ name: string }> };
      expect(data.users).toHaveLength(userCount);
      data.users.forEach((user) => {
        expect(user.name).toBe(`Rapid Update ${updatesPerUser - 1}`);
      });
    }
  });

  it('should handle repeated updates on same 25 users', async () => {
    const db = getDb();
    const userCount = 25;
    const updateRounds = 3;

    // Batch insert
    const users = Array.from({ length: userCount }, (_, i) => ({ name: `Repeat User ${i}`, email: `repeat${i}@example.com` }));
    const [inserted] = await db.query<Array<Array<{ id: { toString: () => string } }>>>(`INSERT INTO user $users RETURN id`, { users });
    const createdUsers = inserted.map((u) => ({ id: u.id.toString() }));

    for (let round = 0; round < updateRounds; round++) {
      const updatePromises = createdUsers.map(({ id }, i) =>
        server.executeOperation(
          {
            query: `mutation UpdateUser($id: ID!, $input: UpdateUserInput!) { updateUser(id: $id, input: $input) { id } }`,
            variables: { id, input: { name: `Repeat User ${i} Round ${round}` } },
          },
          { contextValue: { db } }
        )
      );
      await Promise.all(updatePromises);
    }

    const queryResponse = await server.executeOperation(
      { query: `query GetUsers { users { name } }` },
      { contextValue: { db } }
    );

    expect(queryResponse.body.kind).toBe('single');
    if (queryResponse.body.kind === 'single') {
      const data = queryResponse.body.singleResult.data as { users: Array<{ name: string }> };
      data.users.forEach((user) => {
        expect(user.name).toContain('Round 2');
      });
    }
  });
});
