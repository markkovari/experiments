import { prisma } from '../../../shared/prisma/client';
import { createPostRepository } from '../repository/post-repository';
import { CreatePostInput, Post } from '../domain/types';

// Repository created at module level (singleton pattern)
const postRepository = createPostRepository(prisma);

/**
 * Alternative approach: Direct function with singleton repository
 * No currying, no dependency injection via parameters
 */
export async function createPost(input: CreatePostInput): Promise<Post> {
  // Validation
  if (!input.title || input.title.trim().length === 0) {
    throw new Error('Title is required');
  }

  if (!input.content || input.content.trim().length === 0) {
    throw new Error('Content is required');
  }

  if (!input.authorId) {
    throw new Error('Author ID is required');
  }

  // Create post using singleton repository
  return postRepository.create(input);
}
