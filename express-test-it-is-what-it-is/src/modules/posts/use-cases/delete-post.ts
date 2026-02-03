import { PostRepository } from '../repository/post-repository';
import { NotFoundError } from '../../../shared/middleware/error-handler';

export type DeletePostUseCase = (id: string) => Promise<void>;

export const deletePostUseCase =
  (postRepository: PostRepository): DeletePostUseCase =>
  async (id: string) => {
    const existingPost = await postRepository.findById(id);
    if (!existingPost) throw new NotFoundError('Post');

    return postRepository.delete(id);
  };
