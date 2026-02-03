import { PostRepository } from '../repository/post-repository';
import { CreatePostInput, Post } from '../domain/types';

export type CreatePostUseCase = (input: CreatePostInput) => Promise<Post>;

export const createPostUseCase =
  (postRepository: PostRepository): CreatePostUseCase =>
  async (input: CreatePostInput) => {
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

    // Create post
    return postRepository.create(input);
  };
