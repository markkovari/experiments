import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createPostUseCase } from '../../../src/modules/posts/use-cases/create-post';
import { PostRepository } from '../../../src/modules/posts/repository/post-repository';
import { Post } from '../../../src/modules/posts/domain/types';

describe('CreatePost Use Case - Unit Tests', () => {
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

  it('should create a post with valid input', async () => {
    const input = {
      title: 'Test Post',
      content: 'Test content',
      authorId: 'user-1',
      published: false,
    };

    const createdPost: Post = {
      id: 'post-1',
      ...input,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    vi.mocked(mockRepository.create).mockResolvedValue(createdPost);

    const useCase = createPostUseCase(mockRepository);
    const result = await useCase(input);

    expect(mockRepository.create).toHaveBeenCalledWith(input);
    expect(result).toEqual(createdPost);
  });

  it('should throw error when title is empty', async () => {
    const input = {
      title: '',
      content: 'Test content',
      authorId: 'user-1',
    };

    const useCase = createPostUseCase(mockRepository);

    await expect(useCase(input)).rejects.toThrow('Title is required');
    expect(mockRepository.create).not.toHaveBeenCalled();
  });

  it('should throw error when title is only whitespace', async () => {
    const input = {
      title: '   ',
      content: 'Test content',
      authorId: 'user-1',
    };

    const useCase = createPostUseCase(mockRepository);

    await expect(useCase(input)).rejects.toThrow('Title is required');
  });

  it('should throw error when content is empty', async () => {
    const input = {
      title: 'Test Post',
      content: '',
      authorId: 'user-1',
    };

    const useCase = createPostUseCase(mockRepository);

    await expect(useCase(input)).rejects.toThrow('Content is required');
  });

  it('should throw error when authorId is missing', async () => {
    const input = {
      title: 'Test Post',
      content: 'Test content',
      authorId: '',
    };

    const useCase = createPostUseCase(mockRepository);

    await expect(useCase(input)).rejects.toThrow('Author ID is required');
  });
});
