/**
 * Utility functions for working with API endpoints
 */

/**
 * Build a path by replacing parameters in a route template
 * @example
 * buildPath('/todos/:id', { id: '123' }) // returns '/todos/123'
 * buildPath('/todos', {}) // returns '/todos'
 */
export function buildPath(
  path: string,
  params?: Record<string, string | number>
): string {
  if (!params) return path;

  return Object.entries(params).reduce(
    (acc, [key, value]) => acc.replace(`:${key}`, String(value)),
    path
  );
}
