import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { PrismaClient } from '@prisma/client';
import { UserRepository } from '../../../src/modules/users/repository/UserRepository';
import { UserService } from '../../../src/modules/users/services/UserService';
import { setupTestDatabase, teardownTestDatabase, cleanDatabase } from '../../setup/testcontainers';

describe('UserService - Integration Tests', () => {
  let prisma: PrismaClient;
  let userRepository: UserRepository;
  let userService: UserService;

  beforeAll(async () => {
    const setup = await setupTestDatabase();
    prisma = setup.prisma;
    userRepository = new UserRepository(prisma);
    userService = new UserService(userRepository);
  }, 60000);

  afterAll(async () => {
    await teardownTestDatabase();
  });

  beforeEach(async () => {
    await cleanDatabase();
  });

  describe('createUser with real database', () => {
    it('should create user and enforce email uniqueness constraint', async () => {
      // First user should succeed
      const user1 = await userService.createUser({
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      expect(user1.email).toBe('test@example.com');

      // Second user with same email should fail due to business logic check
      await expect(
        userService.createUser({
          email: 'test@example.com',
          name: 'Another User',
          password: 'password123',
        })
      ).rejects.toThrow('User with this email already exists');

      // Verify only one user exists in database
      const allUsers = await userService.getAllUsers();
      expect(allUsers).toHaveLength(1);
    });

    it('should persist user data correctly', async () => {
      const userData = {
        email: 'persist@example.com',
        name: 'Persistent User',
        password: 'securepass',
      };

      const created = await userService.createUser(userData);

      // Retrieve from database to verify persistence
      const retrieved = await userService.getUserById(created.id);

      expect(retrieved).not.toBeNull();
      expect(retrieved?.email).toBe(userData.email);
      expect(retrieved?.name).toBe(userData.name);
      expect(retrieved?.password).toBe(userData.password);
    });
  });

  describe('updateUser with real database', () => {
    it('should update user and enforce email uniqueness', async () => {
      // Create two users
      const user1 = await userService.createUser({
        email: 'user1@example.com',
        name: 'User 1',
        password: 'pass1',
      });

      const user2 = await userService.createUser({
        email: 'user2@example.com',
        name: 'User 2',
        password: 'pass2',
      });

      // Try to update user2's email to user1's email (should fail)
      await expect(
        userService.updateUser(user2.id, { email: 'user1@example.com' })
      ).rejects.toThrow('Email already in use');

      // Update with unique email should succeed
      const updated = await userService.updateUser(user2.id, {
        email: 'newemail@example.com',
        name: 'Updated Name',
      });

      expect(updated.email).toBe('newemail@example.com');
      expect(updated.name).toBe('Updated Name');

      // Verify in database
      const fromDb = await userService.getUserById(user2.id);
      expect(fromDb?.email).toBe('newemail@example.com');
    });

    it('should handle non-existent user correctly', async () => {
      await expect(
        userService.updateUser('non-existent-id', { name: 'New Name' })
      ).rejects.toThrow('User not found');
    });
  });

  describe('deleteUser with real database', () => {
    it('should delete user and verify removal', async () => {
      const user = await userService.createUser({
        email: 'todelete@example.com',
        name: 'To Delete',
        password: 'password',
      });

      // User should exist
      const beforeDelete = await userService.getUserById(user.id);
      expect(beforeDelete).not.toBeNull();

      // Delete user
      await userService.deleteUser(user.id);

      // User should not exist
      const afterDelete = await userService.getUserById(user.id);
      expect(afterDelete).toBeNull();

      // Should not be in all users list
      const allUsers = await userService.getAllUsers();
      expect(allUsers.find((u) => u.id === user.id)).toBeUndefined();
    });

    it('should handle deleting non-existent user', async () => {
      await expect(userService.deleteUser('non-existent-id')).rejects.toThrow('User not found');
    });
  });

  describe('complex scenarios with real database', () => {
    it('should handle multiple operations in sequence', async () => {
      // Create
      const created = await userService.createUser({
        email: 'complex@example.com',
        name: 'Original Name',
        password: 'password',
      });

      // Update multiple times
      await userService.updateUser(created.id, { name: 'Updated Once' });
      await userService.updateUser(created.id, { name: 'Updated Twice' });

      // Verify final state
      const final = await userService.getUserById(created.id);
      expect(final?.name).toBe('Updated Twice');
    });

    it('should maintain data consistency with concurrent-like operations', async () => {
      // Create multiple users
      const users = await Promise.all([
        userService.createUser({ email: 'user1@test.com', name: 'User 1', password: 'pass' }),
        userService.createUser({ email: 'user2@test.com', name: 'User 2', password: 'pass' }),
        userService.createUser({ email: 'user3@test.com', name: 'User 3', password: 'pass' }),
      ]);

      expect(users).toHaveLength(3);

      // Verify all users are retrievable
      const allUsers = await userService.getAllUsers();
      expect(allUsers).toHaveLength(3);

      // Each user should have unique email
      const emails = allUsers.map((u) => u.email);
      const uniqueEmails = new Set(emails);
      expect(uniqueEmails.size).toBe(3);
    });
  });
});
