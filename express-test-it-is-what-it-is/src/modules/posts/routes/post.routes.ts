import { Router } from 'express';
import { PrismaClient } from '@prisma/client';
import { createPostRepository } from '../repository/post-repository';
import {
  createPostUseCase,
  getPostByIdUseCase,
  getAllPostsUseCase,
  getPostsByAuthorUseCase,
  getPublishedPostsUseCase,
  updatePostUseCase,
  deletePostUseCase,
} from '../use-cases';
import {
  createGetAllPostsHandler,
  createGetPublishedPostsHandler,
  createGetPostByIdHandler,
  createGetPostsByAuthorHandler,
  createCreatePostHandler,
  createUpdatePostHandler,
  createDeletePostHandler,
} from '../handlers/post-handlers';

export function createPostRoutes(prisma: PrismaClient): Router {
  const router = Router();

  // Create repository
  const postRepository = createPostRepository(prisma);

  // Create use cases
  const getAllPosts = getAllPostsUseCase(postRepository);
  const getPublishedPosts = getPublishedPostsUseCase(postRepository);
  const getPostById = getPostByIdUseCase(postRepository);
  const getPostsByAuthor = getPostsByAuthorUseCase(postRepository);
  const createPost = createPostUseCase(postRepository);
  const updatePost = updatePostUseCase(postRepository);
  const deletePost = deletePostUseCase(postRepository);

  // Create handlers
  const getAllPostsHandler = createGetAllPostsHandler(getAllPosts);
  const getPublishedPostsHandler = createGetPublishedPostsHandler(getPublishedPosts);
  const getPostByIdHandler = createGetPostByIdHandler(getPostById);
  const getPostsByAuthorHandler = createGetPostsByAuthorHandler(getPostsByAuthor);
  const createPostHandler = createCreatePostHandler(createPost);
  const updatePostHandler = createUpdatePostHandler(updatePost);
  const deletePostHandler = createDeletePostHandler(deletePost);

  // Define routes
  router.get('/', getAllPostsHandler);
  router.get('/published', getPublishedPostsHandler);
  router.get('/:id', getPostByIdHandler);
  router.get('/author/:authorId', getPostsByAuthorHandler);
  router.post('/', createPostHandler);
  router.put('/:id', updatePostHandler);
  router.delete('/:id', deletePostHandler);

  return router;
}
