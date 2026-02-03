import { PostRepository } from '../repository/post-repository';

export type DeletePostUseCase = (id: string) => Promise<void>;

export const deletePostUseCase =
  (postRepository: PostRepository): DeletePostUseCase =>
  async (id: string) => {
    if (!id) {
      throw new Error('Post ID is required');
    }

    // Check if post exists
    const existingPost = await postRepository.findById(id);
    if (!existingPost) {
      throw new Error('Post not found');
    }

    return postRepository.delete(id);
  };
