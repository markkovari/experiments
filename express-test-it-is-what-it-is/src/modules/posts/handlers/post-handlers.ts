import { Request, Response } from 'express';
import { toPostDTO } from '../domain/types';
import {
  CreatePostUseCase,
  GetPostByIdUseCase,
  GetAllPostsUseCase,
  GetPostsByAuthorUseCase,
  GetPublishedPostsUseCase,
  UpdatePostUseCase,
  DeletePostUseCase,
} from '../use-cases';
import { ValidatedRequest } from '../../../shared/middleware/validation';
import {
  createPostSchema,
  updatePostSchema,
  postIdParamSchema,
  authorIdParamSchema,
} from '../schemas/post.schemas';
import { NotFoundError } from '../../../shared/middleware/error-handler';

export const createGetAllPostsHandler = (getAllPosts: GetAllPostsUseCase) => {
  return async (_req: Request, res: Response): Promise<void> => {
    const posts = await getAllPosts();
    res.json(posts.map(toPostDTO));
  };
};

export const createGetPublishedPostsHandler = (getPublishedPosts: GetPublishedPostsUseCase) => {
  return async (_req: Request, res: Response): Promise<void> => {
    const posts = await getPublishedPosts();
    res.json(posts.map(toPostDTO));
  };
};

export const createGetPostByIdHandler = (getPostById: GetPostByIdUseCase) => {
  return async (req: ValidatedRequest<any, typeof postIdParamSchema>, res: Response): Promise<void> => {
    const { id } = req.params;
    const post = await getPostById(id);
    if (!post) throw new NotFoundError('Post');
    res.json(toPostDTO(post));
  };
};

export const createGetPostsByAuthorHandler = (getPostsByAuthor: GetPostsByAuthorUseCase) => {
  return async (req: ValidatedRequest<any, typeof authorIdParamSchema>, res: Response): Promise<void> => {
    const { authorId } = req.params;
    const posts = await getPostsByAuthor(authorId);
    res.json(posts.map(toPostDTO));
  };
};

export const createCreatePostHandler = (createPost: CreatePostUseCase) => {
  return async (req: ValidatedRequest<typeof createPostSchema>, res: Response): Promise<void> => {
    const { title, content, authorId, published } = req.body;
    const post = await createPost({ title, content, authorId, published });
    res.status(201).json(toPostDTO(post));
  };
};

export const createUpdatePostHandler = (updatePost: UpdatePostUseCase) => {
  return async (req: ValidatedRequest<typeof updatePostSchema, typeof postIdParamSchema>, res: Response): Promise<void> => {
    const { id } = req.params;
    const { title, content, published } = req.body;
    const post = await updatePost(id, { title, content, published });
    res.json(toPostDTO(post));
  };
};

export const createDeletePostHandler = (deletePost: DeletePostUseCase) => {
  return async (req: ValidatedRequest<any, typeof postIdParamSchema>, res: Response): Promise<void> => {
    const { id } = req.params;
    await deletePost(id);
    res.status(204).send();
  };
};
