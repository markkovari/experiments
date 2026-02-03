import { describe, it, expect, vi, beforeEach } from 'vitest';
import { UserService } from '../../../src/modules/users/services/UserService';
import { IUserRepository } from '../../../src/modules/users/repository/IUserRepository';
import { User } from '../../../src/modules/users/domain/User';

describe('UserService - Unit Tests', () => {
  let userService: UserService;
  let mockRepository: IUserRepository;

  beforeEach(() => {
    // Create mock repository
    mockRepository = {
      findById: vi.fn(),
      findByEmail: vi.fn(),
      findAll: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
    };

    userService = new UserService(mockRepository);
  });

  describe('createUser', () => {
    it('should create a user when email is unique', async () => {
      const userData = {
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      };

      const createdUser = User.create({
        id: '1',
        ...userData,
      });

      vi.mocked(mockRepository.findByEmail).mockResolvedValue(null);
      vi.mocked(mockRepository.create).mockResolvedValue(createdUser);

      const result = await userService.createUser(userData);

      expect(mockRepository.findByEmail).toHaveBeenCalledWith(userData.email);
      expect(mockRepository.create).toHaveBeenCalledWith(userData);
      expect(result).toEqual(createdUser);
    });

    it('should throw error when email already exists', async () => {
      const userData = {
        email: 'existing@example.com',
        name: 'Test User',
        password: 'password123',
      };

      const existingUser = User.create({
        id: '1',
        ...userData,
      });

      vi.mocked(mockRepository.findByEmail).mockResolvedValue(existingUser);

      await expect(userService.createUser(userData)).rejects.toThrow(
        'User with this email already exists'
      );
      expect(mockRepository.create).not.toHaveBeenCalled();
    });
  });

  describe('getUserById', () => {
    it('should return user when found', async () => {
      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      vi.mocked(mockRepository.findById).mockResolvedValue(user);

      const result = await userService.getUserById('1');

      expect(mockRepository.findById).toHaveBeenCalledWith('1');
      expect(result).toEqual(user);
    });

    it('should return null when user not found', async () => {
      vi.mocked(mockRepository.findById).mockResolvedValue(null);

      const result = await userService.getUserById('999');

      expect(result).toBeNull();
    });
  });

  describe('updateUser', () => {
    it('should update user when exists', async () => {
      const existingUser = User.create({
        id: '1',
        email: 'old@example.com',
        name: 'Old Name',
        password: 'password123',
      });

      const updatedUser = User.create({
        id: '1',
        email: 'old@example.com',
        name: 'New Name',
        password: 'password123',
      });

      vi.mocked(mockRepository.findById).mockResolvedValue(existingUser);
      vi.mocked(mockRepository.update).mockResolvedValue(updatedUser);

      const result = await userService.updateUser('1', { name: 'New Name' });

      expect(mockRepository.findById).toHaveBeenCalledWith('1');
      expect(mockRepository.update).toHaveBeenCalledWith('1', { name: 'New Name' });
      expect(result.name).toBe('New Name');
    });

    it('should throw error when user not found', async () => {
      vi.mocked(mockRepository.findById).mockResolvedValue(null);

      await expect(userService.updateUser('999', { name: 'New Name' })).rejects.toThrow(
        'User not found'
      );
    });

    it('should throw error when email already in use', async () => {
      const existingUser = User.create({
        id: '1',
        email: 'user1@example.com',
        name: 'User 1',
        password: 'password123',
      });

      const anotherUser = User.create({
        id: '2',
        email: 'user2@example.com',
        name: 'User 2',
        password: 'password123',
      });

      vi.mocked(mockRepository.findById).mockResolvedValue(existingUser);
      vi.mocked(mockRepository.findByEmail).mockResolvedValue(anotherUser);

      await expect(
        userService.updateUser('1', { email: 'user2@example.com' })
      ).rejects.toThrow('Email already in use');
    });
  });

  describe('deleteUser', () => {
    it('should delete user when exists', async () => {
      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      vi.mocked(mockRepository.findById).mockResolvedValue(user);
      vi.mocked(mockRepository.delete).mockResolvedValue(undefined);

      await userService.deleteUser('1');

      expect(mockRepository.findById).toHaveBeenCalledWith('1');
      expect(mockRepository.delete).toHaveBeenCalledWith('1');
    });

    it('should throw error when user not found', async () => {
      vi.mocked(mockRepository.findById).mockResolvedValue(null);

      await expect(userService.deleteUser('999')).rejects.toThrow('User not found');
      expect(mockRepository.delete).not.toHaveBeenCalled();
    });
  });
});
