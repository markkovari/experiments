import { prisma } from '../../../shared/prisma/client';
import { createPostRepository } from '../repository/post-repository';
import { Post } from '../domain/types';

// Repository created at module level (singleton pattern)
const postRepository = createPostRepository(prisma);

/**
 * Alternative approach: Direct functions with singleton repository
 * No currying, no dependency injection via parameters
 */

export async function getPostById(id: string): Promise<Post | null> {
  if (!id) {
    throw new Error('Post ID is required');
  }
  return postRepository.findById(id);
}

export async function getAllPosts(): Promise<Post[]> {
  return postRepository.findAll();
}

export async function getPostsByAuthor(authorId: string): Promise<Post[]> {
  if (!authorId) {
    throw new Error('Author ID is required');
  }
  return postRepository.findByAuthorId(authorId);
}

export async function getPublishedPosts(): Promise<Post[]> {
  return postRepository.findPublished();
}
