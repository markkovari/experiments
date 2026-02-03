import { PostRepository } from '../repository/post-repository';
import { UpdatePostInput, Post } from '../domain/types';

export type UpdatePostUseCase = (id: string, input: UpdatePostInput) => Promise<Post>;

export const updatePostUseCase =
  (postRepository: PostRepository): UpdatePostUseCase =>
  async (id: string, input: UpdatePostInput) => {
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
  };
