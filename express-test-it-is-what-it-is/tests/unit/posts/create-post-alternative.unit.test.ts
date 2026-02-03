import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { Post } from '../../../src/modules/posts/domain/types';

/**
 * Unit tests for alternative approach (direct functions with singleton)
 *
 * Challenge: Harder to test because dependency is at module level
 * Solution: Use vi.doMock() to mock the module before importing
 */

describe('CreatePost Alternative - Unit Tests', () => {
  let createPost: (input: any) => Promise<Post>;
  let mockRepository: any;

  beforeEach(async () => {
    // Create mock repository
    mockRepository = {
      create: vi.fn(),
      findById: vi.fn(),
      findAll: vi.fn(),
      findByAuthorId: vi.fn(),
      findPublished: vi.fn(),
      update: vi.fn(),
      delete: vi.fn(),
    };

    // Mock the repository factory to return our mock
    vi.doMock('../../../src/modules/posts/repository/post-repository', () => ({
      createPostRepository: vi.fn(() => mockRepository),
    }));

    // Mock prisma client
    vi.doMock('../../../src/shared/prisma/client', () => ({
      prisma: {},
    }));

    // Now import the use case (it will use our mocked dependencies)
    const module = await import('../../../src/modules/posts/use-cases-alternative/create-post');
    createPost = module.createPost;
  });

  afterEach(() => {
    vi.doUnmock('../../../src/modules/posts/repository/post-repository');
    vi.doUnmock('../../../src/shared/prisma/client');
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

    mockRepository.create.mockResolvedValue(createdPost);

    const result = await createPost(input);

    expect(mockRepository.create).toHaveBeenCalledWith(input);
    expect(result).toEqual(createdPost);
  });

  it('should throw error when title is empty', async () => {
    const input = {
      title: '',
      content: 'Test content',
      authorId: 'user-1',
    };

    await expect(createPost(input)).rejects.toThrow('Title is required');
    expect(mockRepository.create).not.toHaveBeenCalled();
  });

  it('should throw error when content is empty', async () => {
    const input = {
      title: 'Test Post',
      content: '',
      authorId: 'user-1',
    };

    await expect(createPost(input)).rejects.toThrow('Content is required');
  });

  it('should throw error when authorId is missing', async () => {
    const input = {
      title: 'Test Post',
      content: 'Content',
      authorId: '',
    };

    await expect(createPost(input)).rejects.toThrow('Author ID is required');
  });
});
