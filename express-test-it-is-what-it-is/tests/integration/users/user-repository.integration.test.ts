import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { PrismaClient } from '@prisma/client';
import { UserRepository } from '../../../src/modules/users/repository/UserRepository';
import { setupTestDatabase, teardownTestDatabase, cleanDatabase } from '../../setup/testcontainers';

describe('UserRepository - Integration Tests', () => {
  let prisma: PrismaClient;
  let userRepository: UserRepository;

  beforeAll(async () => {
    const setup = await setupTestDatabase();
    prisma = setup.prisma;
    userRepository = new UserRepository(prisma);
  }, 60000);

  afterAll(async () => {
    await teardownTestDatabase();
  });

  beforeEach(async () => {
    await cleanDatabase();
  });

  describe('create', () => {
    it('should create a user in the database', async () => {
      const userData = {
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      };

      const user = await userRepository.create(userData);

      expect(user.id).toBeDefined();
      expect(user.email).toBe(userData.email);
      expect(user.name).toBe(userData.name);
      expect(user.password).toBe(userData.password);
      expect(user.createdAt).toBeInstanceOf(Date);
      expect(user.updatedAt).toBeInstanceOf(Date);
    });
  });

  describe('findById', () => {
    it('should find a user by id', async () => {
      const createdUser = await userRepository.create({
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      const foundUser = await userRepository.findById(createdUser.id);

      expect(foundUser).toBeDefined();
      expect(foundUser?.id).toBe(createdUser.id);
      expect(foundUser?.email).toBe(createdUser.email);
    });

    it('should return null when user not found', async () => {
      const user = await userRepository.findById('non-existent-id');
      expect(user).toBeNull();
    });
  });

  describe('findByEmail', () => {
    it('should find a user by email', async () => {
      const createdUser = await userRepository.create({
        email: 'unique@example.com',
        name: 'Test User',
        password: 'password123',
      });

      const foundUser = await userRepository.findByEmail('unique@example.com');

      expect(foundUser).toBeDefined();
      expect(foundUser?.id).toBe(createdUser.id);
      expect(foundUser?.email).toBe('unique@example.com');
    });

    it('should return null when user not found', async () => {
      const user = await userRepository.findByEmail('nonexistent@example.com');
      expect(user).toBeNull();
    });
  });

  describe('findAll', () => {
    it('should return all users', async () => {
      await userRepository.create({
        email: 'user1@example.com',
        name: 'User 1',
        password: 'password123',
      });

      await userRepository.create({
        email: 'user2@example.com',
        name: 'User 2',
        password: 'password123',
      });

      const users = await userRepository.findAll();

      expect(users).toHaveLength(2);
      expect(users.some((u) => u.email === 'user1@example.com')).toBe(true);
      expect(users.some((u) => u.email === 'user2@example.com')).toBe(true);
    });

    it('should return empty array when no users exist', async () => {
      const users = await userRepository.findAll();
      expect(users).toEqual([]);
    });
  });

  describe('update', () => {
    it('should update a user', async () => {
      const createdUser = await userRepository.create({
        email: 'test@example.com',
        name: 'Old Name',
        password: 'password123',
      });

      const updatedUser = await userRepository.update(createdUser.id, {
        name: 'New Name',
      });

      expect(updatedUser.id).toBe(createdUser.id);
      expect(updatedUser.name).toBe('New Name');
      expect(updatedUser.email).toBe('test@example.com');
    });

    it('should update user email', async () => {
      const createdUser = await userRepository.create({
        email: 'old@example.com',
        name: 'Test User',
        password: 'password123',
      });

      const updatedUser = await userRepository.update(createdUser.id, {
        email: 'new@example.com',
      });

      expect(updatedUser.email).toBe('new@example.com');
    });
  });

  describe('delete', () => {
    it('should delete a user', async () => {
      const createdUser = await userRepository.create({
        email: 'todelete@example.com',
        name: 'Test User',
        password: 'password123',
      });

      await userRepository.delete(createdUser.id);

      const foundUser = await userRepository.findById(createdUser.id);
      expect(foundUser).toBeNull();
    });
  });
});
