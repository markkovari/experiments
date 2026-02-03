import { Router } from 'express';
import {
  handleGetAllPosts,
  handleGetPublishedPosts,
  handleGetPostById,
  handleGetPostsByAuthor,
  handleCreatePost,
  handleUpdatePost,
  handleDeletePost,
} from '../handlers-alternative/post-handlers';

/**
 * Alternative approach: Direct route configuration
 * Handlers are imported and used directly (no factory functions)
 * Dependencies are resolved at module level (singleton pattern)
 */
export function createPostRoutesAlternative(): Router {
  const router = Router();

  // Define routes with direct handler references
  router.get('/', handleGetAllPosts);
  router.get('/published', handleGetPublishedPosts);
  router.get('/:id', handleGetPostById);
  router.get('/author/:authorId', handleGetPostsByAuthor);
  router.post('/', handleCreatePost);
  router.put('/:id', handleUpdatePost);
  router.delete('/:id', handleDeletePost);

  return router;
}
