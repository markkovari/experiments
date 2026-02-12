"use strict";
/**
 * Utility functions for working with API endpoints
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildPath = buildPath;
/**
 * Build a path by replacing parameters in a route template
 * @example
 * buildPath('/todos/:id', { id: '123' }) // returns '/todos/123'
 * buildPath('/todos', {}) // returns '/todos'
 */
function buildPath(path, params) {
    if (!params)
        return path;
    return Object.entries(params).reduce((acc, [key, value]) => acc.replace(`:${key}`, String(value)), path);
}
