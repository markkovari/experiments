import { prisma } from '../../../shared/prisma/client';
import { createPostRepository } from '../repository/post-repository';
import { UpdatePostInput, Post } from '../domain/types';

// Repository created at module level (singleton pattern)
const postRepository = createPostRepository(prisma);

/**
 * Alternative approach: Direct function with singleton repository
 * No currying, no dependency injection via parameters
 */
export async function updatePost(id: string, input: UpdatePostInput): Promise<Post> {
  if (!id) {
    throw new Error('Post ID is required');
  }

  // Check if post exists
  const existingPost = await postRepository.findById(id);
  if (!existingPost) {
    throw new Error('Post not found');
  }

  // Validate input
  if (input.title !== undefined && input.title.trim().length === 0) {
    throw new Error('Title cannot be empty');
  }

  if (input.content !== undefined && input.content.trim().length === 0) {
    throw new Error('Content cannot be empty');
  }

  return postRepository.update(id, input);
}
