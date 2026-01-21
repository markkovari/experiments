import { ApolloServer } from '@apollo/server';
import { describe, it, expect, beforeEach } from 'vitest';
import { resolvers, type Context } from '../../src/resolvers/index.js';
import { typeDefs } from '../../src/schema/index.js';
import { getDb } from '../setup.js';

describe('User Workflows - Realistic E2E Tests', () => {
  let server: ApolloServer<Context>;

  beforeEach(async () => {
    const db = getDb();
    await db.query('DELETE user');
    server = new ApolloServer<Context>({ typeDefs, resolvers });
  });

  describe('User Registration Flow', () => {
    it('should register a new user and retrieve their profile', async () => {
      const db = getDb();

      // User registers
      const registerResponse = await server.executeOperation(
        {
          query: `
            mutation RegisterUser($input: CreateUserInput!) {
              createUser(input: $input) {
                id
                name
                email
              }
            }
          `,
          variables: {
            input: {
              name: 'Alice Johnson',
              email: 'alice.johnson@example.com',
            },
          },
        },
        { contextValue: { db } }
      );

      expect(registerResponse.body.kind).toBe('single');
      if (registerResponse.body.kind === 'single') {
        expect(registerResponse.body.singleResult.errors).toBeUndefined();
        const data = registerResponse.body.singleResult.data as {
          createUser: { id: string; name: string; email: string };
        };
        expect(data.createUser.name).toBe('Alice Johnson');
        expect(data.createUser.email).toBe('alice.johnson@example.com');

        // User retrieves their profile
        const profileResponse = await server.executeOperation(
          {
            query: `
              query GetProfile($id: ID!) {
                user(id: $id) {
                  id
                  name
                  email
                }
              }
            `,
            variables: { id: data.createUser.id },
          },
          { contextValue: { db } }
        );

        expect(profileResponse.body.kind).toBe('single');
        if (profileResponse.body.kind === 'single') {
          const profileData = profileResponse.body.singleResult.data as {
            user: { id: string; name: string; email: string };
          };
          expect(profileData.user.name).toBe('Alice Johnson');
          expect(profileData.user.email).toBe('alice.johnson@example.com');
        }
      }
    });

    it('should allow user to update their profile information', async () => {
      const db = getDb();

      // Create user
      const createResponse = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: 'Bob Smith', email: 'bob@example.com' } },
        },
        { contextValue: { db } }
      );

      expect(createResponse.body.kind).toBe('single');
      if (createResponse.body.kind === 'single') {
        const userId = (createResponse.body.singleResult.data as { createUser: { id: string } }).createUser.id;

        // User updates their name
        const updateNameResponse = await server.executeOperation(
          {
            query: `
              mutation UpdateProfile($id: ID!, $input: UpdateUserInput!) {
                updateUser(id: $id, input: $input) {
                  id
                  name
                  email
                }
              }
            `,
            variables: { id: userId, input: { name: 'Robert Smith' } },
          },
          { contextValue: { db } }
        );

        expect(updateNameResponse.body.kind).toBe('single');
        if (updateNameResponse.body.kind === 'single') {
          const data = updateNameResponse.body.singleResult.data as {
            updateUser: { name: string; email: string };
          };
          expect(data.updateUser.name).toBe('Robert Smith');
          expect(data.updateUser.email).toBe('bob@example.com'); // Email unchanged
        }

        // User updates their email
        const updateEmailResponse = await server.executeOperation(
          {
            query: `
              mutation UpdateProfile($id: ID!, $input: UpdateUserInput!) {
                updateUser(id: $id, input: $input) {
                  id
                  name
                  email
                }
              }
            `,
            variables: { id: userId, input: { email: 'robert.smith@company.com' } },
          },
          { contextValue: { db } }
        );

        expect(updateEmailResponse.body.kind).toBe('single');
        if (updateEmailResponse.body.kind === 'single') {
          const data = updateEmailResponse.body.singleResult.data as {
            updateUser: { name: string; email: string };
          };
          expect(data.updateUser.name).toBe('Robert Smith');
          expect(data.updateUser.email).toBe('robert.smith@company.com');
        }
      }
    });
  });

  describe('User Directory Operations', () => {
    it('should list all users in the system', async () => {
      const db = getDb();

      // Create multiple users
      const users = [
        { name: 'Charlie Brown', email: 'charlie@example.com' },
        { name: 'Diana Prince', email: 'diana@example.com' },
        { name: 'Edward Norton', email: 'edward@example.com' },
      ];

      for (const user of users) {
        await server.executeOperation(
          {
            query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
            variables: { input: user },
          },
          { contextValue: { db } }
        );
      }

      // List all users
      const listResponse = await server.executeOperation(
        {
          query: `
            query ListUsers {
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

      expect(listResponse.body.kind).toBe('single');
      if (listResponse.body.kind === 'single') {
        const data = listResponse.body.singleResult.data as {
          users: Array<{ id: string; name: string; email: string }>;
        };
        expect(data.users).toHaveLength(3);
        const names = data.users.map((u) => u.name).sort();
        expect(names).toEqual(['Charlie Brown', 'Diana Prince', 'Edward Norton']);
      }
    });

    it('should find a specific user by their ID', async () => {
      const db = getDb();

      // Create a user
      const createResponse = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id name email } }`,
          variables: { input: { name: 'Frank Castle', email: 'frank@example.com' } },
        },
        { contextValue: { db } }
      );

      expect(createResponse.body.kind).toBe('single');
      if (createResponse.body.kind === 'single') {
        const created = (createResponse.body.singleResult.data as { createUser: { id: string } }).createUser;

        // Find user by ID
        const findResponse = await server.executeOperation(
          {
            query: `query FindUser($id: ID!) { user(id: $id) { id name email } }`,
            variables: { id: created.id },
          },
          { contextValue: { db } }
        );

        expect(findResponse.body.kind).toBe('single');
        if (findResponse.body.kind === 'single') {
          const data = findResponse.body.singleResult.data as { user: { name: string; email: string } };
          expect(data.user.name).toBe('Frank Castle');
          expect(data.user.email).toBe('frank@example.com');
        }
      }
    });
  });

  describe('Account Deletion Flow', () => {
    it('should delete a user account and confirm removal', async () => {
      const db = getDb();

      // Create user
      const createResponse = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id } }`,
          variables: { input: { name: 'Grace Hopper', email: 'grace@example.com' } },
        },
        { contextValue: { db } }
      );

      expect(createResponse.body.kind).toBe('single');
      if (createResponse.body.kind === 'single') {
        const userId = (createResponse.body.singleResult.data as { createUser: { id: string } }).createUser.id;

        // Delete account
        const deleteResponse = await server.executeOperation(
          {
            query: `mutation DeleteAccount($id: ID!) { deleteUser(id: $id) }`,
            variables: { id: userId },
          },
          { contextValue: { db } }
        );

        expect(deleteResponse.body.kind).toBe('single');
        if (deleteResponse.body.kind === 'single') {
          expect((deleteResponse.body.singleResult.data as { deleteUser: boolean }).deleteUser).toBe(true);
        }

        // Verify user no longer exists
        const verifyResponse = await server.executeOperation(
          {
            query: `query FindUser($id: ID!) { user(id: $id) { id } }`,
            variables: { id: userId },
          },
          { contextValue: { db } }
        );

        expect(verifyResponse.body.kind).toBe('single');
        if (verifyResponse.body.kind === 'single') {
          expect((verifyResponse.body.singleResult.data as { user: null }).user).toBeNull();
        }
      }
    });

    it('should handle deletion of non-existent user gracefully', async () => {
      const db = getDb();

      const deleteResponse = await server.executeOperation(
        {
          query: `mutation DeleteAccount($id: ID!) { deleteUser(id: $id) }`,
          variables: { id: 'user:nonexistent123' },
        },
        { contextValue: { db } }
      );

      expect(deleteResponse.body.kind).toBe('single');
      if (deleteResponse.body.kind === 'single') {
        // Should return false or handle gracefully
        const data = deleteResponse.body.singleResult.data as { deleteUser: boolean };
        expect(typeof data.deleteUser).toBe('boolean');
      }
    });
  });

  describe('Edge Cases', () => {
    it('should handle user with special characters in name', async () => {
      const db = getDb();

      const response = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id name email } }`,
          variables: { input: { name: "O'Connor-Smith Jr.", email: 'oconnor@example.com' } },
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        const data = response.body.singleResult.data as { createUser: { name: string } };
        expect(data.createUser.name).toBe("O'Connor-Smith Jr.");
      }
    });

    it('should handle user with unicode characters', async () => {
      const db = getDb();

      const response = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id name email } }`,
          variables: { input: { name: '田中太郎', email: 'tanaka@example.jp' } },
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        const data = response.body.singleResult.data as { createUser: { name: string } };
        expect(data.createUser.name).toBe('田中太郎');
      }
    });

    it('should handle email with valid special formats', async () => {
      const db = getDb();

      const response = await server.executeOperation(
        {
          query: `mutation CreateUser($input: CreateUserInput!) { createUser(input: $input) { id email } }`,
          variables: { input: { name: 'Test User', email: 'user+tag@sub.domain.example.com' } },
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        const data = response.body.singleResult.data as { createUser: { email: string } };
        expect(data.createUser.email).toBe('user+tag@sub.domain.example.com');
      }
    });

    it('should return null when querying non-existent user', async () => {
      const db = getDb();

      const response = await server.executeOperation(
        {
          query: `query FindUser($id: ID!) { user(id: $id) { id name } }`,
          variables: { id: 'user:does_not_exist_12345' },
        },
        { contextValue: { db } }
      );

      expect(response.body.kind).toBe('single');
      if (response.body.kind === 'single') {
        expect(response.body.singleResult.errors).toBeUndefined();
        expect((response.body.singleResult.data as { user: null }).user).toBeNull();
      }
    });
  });
});
