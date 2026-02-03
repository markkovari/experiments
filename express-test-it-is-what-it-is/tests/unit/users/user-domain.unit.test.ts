import { describe, it, expect } from 'vitest';
import { User } from '../../../src/modules/users/domain/User';

describe('User Domain - Unit Tests', () => {
  describe('User.create', () => {
    it('should create a user with provided dates', () => {
      const createdAt = new Date('2024-01-01');
      const updatedAt = new Date('2024-01-02');

      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
        createdAt,
        updatedAt,
      });

      expect(user.id).toBe('1');
      expect(user.email).toBe('test@example.com');
      expect(user.name).toBe('Test User');
      expect(user.password).toBe('password123');
      expect(user.createdAt).toEqual(createdAt);
      expect(user.updatedAt).toEqual(updatedAt);
    });

    it('should create a user with default dates when not provided', () => {
      const before = new Date();
      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });
      const after = new Date();

      expect(user.createdAt.getTime()).toBeGreaterThanOrEqual(before.getTime());
      expect(user.createdAt.getTime()).toBeLessThanOrEqual(after.getTime());
      expect(user.updatedAt.getTime()).toBeGreaterThanOrEqual(before.getTime());
      expect(user.updatedAt.getTime()).toBeLessThanOrEqual(after.getTime());
    });
  });

  describe('isPasswordValid', () => {
    it('should return true when password matches', () => {
      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      expect(user.isPasswordValid('password123')).toBe(true);
    });

    it('should return false when password does not match', () => {
      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
      });

      expect(user.isPasswordValid('wrongpassword')).toBe(false);
    });
  });

  describe('toDTO', () => {
    it('should return user data without password', () => {
      const user = User.create({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        password: 'password123',
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-02'),
      });

      const dto = user.toDTO();

      expect(dto).toEqual({
        id: '1',
        email: 'test@example.com',
        name: 'Test User',
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-02'),
      });
      expect(dto).not.toHaveProperty('password');
    });
  });
});
