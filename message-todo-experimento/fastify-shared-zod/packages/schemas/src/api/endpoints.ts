/**
 * API endpoint definitions
 * Provides a single source of truth for API routes
 */
export const TodoEndpoints = {
  list: {
    method: 'GET' as const,
    path: '/todos',
  },
  add: {
    method: 'POST' as const,
    path: '/todos',
  },
  toggle: {
    method: 'PATCH' as const,
    path: '/todos/:id/toggle',
  },
  update: {
    method: 'PUT' as const,
    path: '/todos/:id',
  },
  delete: {
    method: 'DELETE' as const,
    path: '/todos/:id',
  },
} as const;
