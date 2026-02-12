/**
 * Utility functions for working with API endpoints
 */
/**
 * Build a path by replacing parameters in a route template
 * @example
 * buildPath('/todos/:id', { id: '123' }) // returns '/todos/123'
 * buildPath('/todos', {}) // returns '/todos'
 */
export declare function buildPath(path: string, params?: Record<string, string | number>): string;
