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

// Handler factory functions (functional approach)

export const createGetAllPostsHandler = (getAllPosts: GetAllPostsUseCase) => {
  return async (req: Request, res: Response): Promise<void> => {
    try {
      const posts = await getAllPosts();
      res.json(posts.map(toPostDTO));
    } catch (error) {
      res.status(500).json({ error: 'Internal server error' });
    }
  };
};

export const createGetPublishedPostsHandler = (getPublishedPosts: GetPublishedPostsUseCase) => {
  return async (req: Request, res: Response): Promise<void> => {
    try {
      const posts = await getPublishedPosts();
      res.json(posts.map(toPostDTO));
    } catch (error) {
      res.status(500).json({ error: 'Internal server error' });
    }
  };
};

export const createGetPostByIdHandler = (getPostById: GetPostByIdUseCase) => {
  return async (req: Request, res: Response): Promise<void> => {
    try {
      const { id } = req.params;
      const post = await getPostById(id);

      if (!post) {
        res.status(404).json({ error: 'Post not found' });
        return;
      }

      res.json(toPostDTO(post));
    } catch (error) {
      res.status(500).json({ error: 'Internal server error' });
    }
  };
};

export const createGetPostsByAuthorHandler = (getPostsByAuthor: GetPostsByAuthorUseCase) => {
  return async (req: Request, res: Response): Promise<void> => {
    try {
      const { authorId } = req.params;
      const posts = await getPostsByAuthor(authorId);
      res.json(posts.map(toPostDTO));
    } catch (error) {
      res.status(500).json({ error: 'Internal server error' });
    }
  };
};

export const createCreatePostHandler = (createPost: CreatePostUseCase) => {
  return async (req: Request, res: Response): Promise<void> => {
    try {
      const { title, content, authorId, published } = req.body;
      const post = await createPost({ title, content, authorId, published });
      res.status(201).json(toPostDTO(post));
    } catch (error) {
      if (error instanceof Error) {
        res.status(400).json({ error: error.message });
        return;
      }
      res.status(500).json({ error: 'Internal server error' });
    }
  };
};

export const createUpdatePostHandler = (updatePost: UpdatePostUseCase) => {
  return async (req: Request, res: Response): Promise<void> => {
    try {
      const { id } = req.params;
      const { title, content, published } = req.body;
      const post = await updatePost(id, { title, content, published });
      res.json(toPostDTO(post));
    } catch (error) {
      if (error instanceof Error && error.message === 'Post not found') {
        res.status(404).json({ error: error.message });
        return;
      }
      if (error instanceof Error) {
        res.status(400).json({ error: error.message });
        return;
      }
      res.status(500).json({ error: 'Internal server error' });
    }
  };
};

export const createDeletePostHandler = (deletePost: DeletePostUseCase) => {
  return async (req: Request, res: Response): Promise<void> => {
    try {
      const { id } = req.params;
      await deletePost(id);
      res.status(204).send();
    } catch (error) {
      if (error instanceof Error && error.message === 'Post not found') {
        res.status(404).json({ error: error.message });
        return;
      }
      res.status(500).json({ error: 'Internal server error' });
    }
  };
};
