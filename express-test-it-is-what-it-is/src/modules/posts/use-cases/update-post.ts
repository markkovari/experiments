import { PostRepository } from '../repository/post-repository';
import { UpdatePostInput, Post } from '../domain/types';
import { NotFoundError } from '../../../shared/middleware/error-handler';

export type UpdatePostUseCase = (id: string, input: UpdatePostInput) => Promise<Post>;

export const updatePostUseCase =
  (postRepository: PostRepository): UpdatePostUseCase =>
  async (id: string, input: UpdatePostInput) => {
    const existingPost = await postRepository.findById(id);
    if (!existingPost) throw new NotFoundError('Post');

    return postRepository.update(id, input);
  };
