import { describe, it, expect, vi, beforeEach } from 'vitest';
import { updatePostUseCase } from '../../../src/modules/posts/use-cases/update-post';
import { PostRepository } from '../../../src/modules/posts/repository/post-repository';
import { Post } from '../../../src/modules/posts/domain/types';

describe('UpdatePost Use Case - Unit Tests', () => {
  let mockRepository: PostRepository;

  beforeEach(() => {
    mockRepository = {
      findById: vi.fn(),
      findAll: vi.fn(),
      findByAuthorId: vi.fn(),
      findPublished: vi.fn(),
      create: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
    };
  });

  it('should update a post when it exists', async () => {
    const existingPost: Post = {
      id: 'post-1',
      title: 'Old Title',
      content: 'Old content',
      published: false,
      authorId: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    const updatedPost: Post = {
      ...existingPost,
      title: 'New Title',
    };

    vi.mocked(mockRepository.findById).mockResolvedValue(existingPost);
    vi.mocked(mockRepository.update).mockResolvedValue(updatedPost);

    const useCase = updatePostUseCase(mockRepository);
    const result = await useCase('post-1', { title: 'New Title' });

    expect(mockRepository.findById).toHaveBeenCalledWith('post-1');
    expect(mockRepository.update).toHaveBeenCalledWith('post-1', { title: 'New Title' });
    expect(result.title).toBe('New Title');
  });

  it('should throw error when post not found', async () => {
    vi.mocked(mockRepository.findById).mockResolvedValue(null);

    const useCase = updatePostUseCase(mockRepository);

    await expect(useCase('post-999', { title: 'New Title' })).rejects.toThrow('Post not found');
    expect(mockRepository.update).not.toHaveBeenCalled();
  });

  it('should throw error when title is empty string', async () => {
    const existingPost: Post = {
      id: 'post-1',
      title: 'Old Title',
      content: 'Old content',
      published: false,
      authorId: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    vi.mocked(mockRepository.findById).mockResolvedValue(existingPost);

    const useCase = updatePostUseCase(mockRepository);

    await expect(useCase('post-1', { title: '' })).rejects.toThrow('Title cannot be empty');
  });

  it('should throw error when content is empty string', async () => {
    const existingPost: Post = {
      id: 'post-1',
      title: 'Old Title',
      content: 'Old content',
      published: false,
      authorId: 'user-1',
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    vi.mocked(mockRepository.findById).mockResolvedValue(existingPost);

    const useCase = updatePostUseCase(mockRepository);

    await expect(useCase('post-1', { content: '' })).rejects.toThrow('Content cannot be empty');
  });
});
