import { PostRepository } from '../repository/post-repository';
import { Post } from '../domain/types';

export type GetPostByIdUseCase = (id: string) => Promise<Post | null>;
export type GetAllPostsUseCase = () => Promise<Post[]>;
export type GetPostsByAuthorUseCase = (authorId: string) => Promise<Post[]>;
export type GetPublishedPostsUseCase = () => Promise<Post[]>;

export const getPostByIdUseCase =
  (postRepository: PostRepository): GetPostByIdUseCase =>
  async (id: string) => {
    if (!id) {
      throw new Error('Post ID is required');
    }
    return postRepository.findById(id);
  };

export const getAllPostsUseCase =
  (postRepository: PostRepository): GetAllPostsUseCase =>
  async () => {
    return postRepository.findAll();
  };

export const getPostsByAuthorUseCase =
  (postRepository: PostRepository): GetPostsByAuthorUseCase =>
  async (authorId: string) => {
    if (!authorId) {
      throw new Error('Author ID is required');
    }
    return postRepository.findByAuthorId(authorId);
  };

export const getPublishedPostsUseCase =
  (postRepository: PostRepository): GetPublishedPostsUseCase =>
  async () => {
    return postRepository.findPublished();
  };
