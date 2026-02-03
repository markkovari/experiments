import { prisma } from '../../../shared/prisma/client';
import { createPostRepository } from '../repository/post-repository';

// Repository created at module level (singleton pattern)
const postRepository = createPostRepository(prisma);

/**
 * Alternative approach: Direct function with singleton repository
 * No currying, no dependency injection via parameters
 */
export async function deletePost(id: string): Promise<void> {
  if (!id) {
    throw new Error('Post ID is required');
  }

  // Check if post exists
  const existingPost = await postRepository.findById(id);
  if (!existingPost) {
    throw new Error('Post not found');
  }

  return postRepository.delete(id);
}
