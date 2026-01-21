import { describe, it, expect, beforeEach } from 'vitest';
import { ApolloServer } from '@apollo/server';
import { typeDefs } from '../../src/schema/index.js';
import { resolvers, type Context } from '../../src/resolvers/index.js';
import { getDb } from '../setup.js';

describe('GraphQL E2E Tests', () => {
  let server: ApolloServer<Context>;

  beforeEach(async () => {
    const db = getDb();
    await db.query('DELETE user');
    server = new ApolloServer<Context>({ typeDefs, resolvers });
  });

  describe('User CRUD Operations', () => {
    it('should create a user', async () => {
      const db = getDb();
      const response = await server.executeOperation(
        {
          query: `
            mutation CreateUser($input: CreateUserInput!) {
              createUser(input: $input) {
                id
                name
                email
              }
            }
          `,
          variables: {
            input: {
              name: 'John Doe',
              email: 'john@example.com',
            },
          },
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        const data = response.body.singleResult.data as { createUser: { id: string; name: string; email: string } };
        expect(data.createUser.name).toBe('John Doe');
        expect(data.createUser.email).toBe('john@example.com');
        expect(data.createUser.id).toBeDefined();
      }
    });

    it('should query all users', async () => {
      const db = getDb();
      await db.create('user', { name: 'User 1', email: 'user1@example.com' });
      await db.create('user', { name: 'User 2', email: 'user2@example.com' });

      const response = await server.executeOperation(
        {
          query: `
            query GetUsers {
              users {
                id
                name
                email
              }
            }
          `,
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        const data = response.body.singleResult.data as { users: Array<{ id: string; name: string; email: string }> };
        expect(data.users).toHaveLength(2);
        expect(data.users.map(u => u.name).sort()).toEqual(['User 1', 'User 2']);
      }
    });

    it('should query a single user by id', async () => {
      const db = getDb();
      const [created] = await db.create('user', { name: 'Test User', email: 'test@example.com' });
      const userId = created.id.toString();

      const response = await server.executeOperation(
        {
          query: `
            query GetUser($id: ID!) {
              user(id: $id) {
                id
                name
                email
              }
            }
          `,
          variables: { id: userId },
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        const data = response.body.singleResult.data as { user: { id: string; name: string; email: string } };
        expect(data.user.name).toBe('Test User');
        expect(data.user.email).toBe('test@example.com');
      }
    });

    it('should update a user', async () => {
      const db = getDb();
      const [created] = await db.create('user', { name: 'Original Name', email: 'original@example.com' });
      const userId = created.id.toString();

      const response = await server.executeOperation(
        {
          query: `
            mutation UpdateUser($id: ID!, $input: UpdateUserInput!) {
              updateUser(id: $id, input: $input) {
                id
                name
                email
              }
            }
          `,
          variables: {
            id: userId,
            input: {
              name: 'Updated Name',
              email: 'updated@example.com',
            },
          },
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        const data = response.body.singleResult.data as { updateUser: { id: string; name: string; email: string } };
        expect(data.updateUser.name).toBe('Updated Name');
        expect(data.updateUser.email).toBe('updated@example.com');
      }
    });

    it('should delete a user', async () => {
      const db = getDb();
      const [created] = await db.create('user', { name: 'To Delete', email: 'delete@example.com' });
      const userId = created.id.toString();

      const deleteResponse = await server.executeOperation(
        {
          query: `
            mutation DeleteUser($id: ID!) {
              deleteUser(id: $id)
            }
          `,
          variables: { id: userId },
        },
        { contextValue: { db } }
      );

      expect(deleteResponse.body.kind).toBe('single');
      if (deleteResponse.body.kind === 'single') {
        expect(deleteResponse.body.singleResult.errors).toBeUndefined();
        const data = deleteResponse.body.singleResult.data as { deleteUser: boolean };
        expect(data.deleteUser).toBe(true);
      }

      const queryResponse = await server.executeOperation(
        {
          query: `
            query GetUser($id: ID!) {
              user(id: $id) {
                id
              }
            }
          `,
          variables: { id: userId },
        },
        { contextValue: { db } }
      );

      expect(queryResponse.body.kind).toBe('single');
      if (queryResponse.body.kind === 'single') {
        const data = queryResponse.body.singleResult.data as { user: null };
        expect(data.user).toBeNull();
      }
    });

    it('should return null for non-existent user', async () => {
      const db = getDb();

      const response = await server.executeOperation(
        {
          query: `
            query GetUser($id: ID!) {
              user(id: $id) {
                id
                name
                email
              }
            }
          `,
          variables: { id: 'user:nonexistent' },
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        const data = response.body.singleResult.data as { user: null };
        expect(data.user).toBeNull();
      }
    });
  });
});
