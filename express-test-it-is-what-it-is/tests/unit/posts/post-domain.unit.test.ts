import { describe, it, expect } from 'vitest';
import { toPostDTO, Post } from '../../../src/modules/posts/domain/types';

describe('Post Domain - Unit Tests', () => {
  describe('toPostDTO', () => {
    it('should convert Post to DTO', () => {
      const post: Post = {
        id: 'post-1',
        title: 'Test Post',
        content: 'Test content',
        published: true,
        authorId: 'user-1',
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-02'),
      };

      const dto = toPostDTO(post);

      expect(dto).toEqual({
        id: 'post-1',
        title: 'Test Post',
        content: 'Test content',
        published: true,
        authorId: 'user-1',
        createdAt: new Date('2024-01-01'),
        updatedAt: new Date('2024-01-02'),
      });
    });

    it('should handle unpublished posts', () => {
      const post: Post = {
        id: 'post-1',
        title: 'Draft Post',
        content: 'Draft content',
        published: false,
        authorId: 'user-1',
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      const dto = toPostDTO(post);

      expect(dto.published).toBe(false);
    });
  });
});
