import { Request, Response } from 'express';
import { toPostDTO } from '../domain/types';
import {
  createPost,
  getPostById,
  getAllPosts,
  getPostsByAuthor,
  getPublishedPosts,
  updatePost,
  deletePost,
} from '../use-cases-alternative';

/**
 * Alternative approach: Direct handler functions
 * Use cases are imported and called directly (no currying)
 */

export async function handleGetAllPosts(req: Request, res: Response): Promise<void> {
  try {
    const posts = await getAllPosts();
    res.json(posts.map(toPostDTO));
  } catch (error) {
    res.status(500).json({ error: 'Internal server error' });
  }
}

export async function handleGetPublishedPosts(req: Request, res: Response): Promise<void> {
  try {
    const posts = await getPublishedPosts();
    res.json(posts.map(toPostDTO));
  } catch (error) {
    res.status(500).json({ error: 'Internal server error' });
  }
}

export async function handleGetPostById(req: Request, res: Response): Promise<void> {
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
}

export async function handleGetPostsByAuthor(req: Request, res: Response): Promise<void> {
  try {
    const { authorId } = req.params;
    const posts = await getPostsByAuthor(authorId);
    res.json(posts.map(toPostDTO));
  } catch (error) {
    res.status(500).json({ error: 'Internal server error' });
  }
}

export async function handleCreatePost(req: Request, res: Response): Promise<void> {
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
}

export async function handleUpdatePost(req: Request, res: Response): Promise<void> {
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
}

export async function handleDeletePost(req: Request, res: Response): Promise<void> {
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
}
