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
import { validate } from '../../../shared/middleware/validation';
import { asyncHandler } from '../../../shared/middleware/async-handler';
import {
  createPostSchema,
  updatePostSchema,
  postIdParamSchema,
  authorIdParamSchema,
} from '../schemas/post.schemas';

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

  // Define routes with validation and async handling
  router.get('/', asyncHandler(getAllPostsHandler));
  router.get('/published', asyncHandler(getPublishedPostsHandler));
  router.get('/:id', validate({ params: postIdParamSchema }), asyncHandler(getPostByIdHandler));
  router.get(
    '/author/:authorId',
    validate({ params: authorIdParamSchema }),
    asyncHandler(getPostsByAuthorHandler),
  );
  router.post('/', validate({ body: createPostSchema }), asyncHandler(createPostHandler));
  router.put(
    '/:id',
    validate({ params: postIdParamSchema, body: updatePostSchema }),
    asyncHandler(updatePostHandler),
  );
  router.delete('/:id', validate({ params: postIdParamSchema }), asyncHandler(deletePostHandler));

  return router;
}
