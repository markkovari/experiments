"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TodoEndpoints = void 0;
/**
 * API endpoint definitions
 * Provides a single source of truth for API routes
 */
exports.TodoEndpoints = {
    list: {
        method: 'GET',
        path: '/todos',
    },
    add: {
        method: 'POST',
        path: '/todos',
    },
    toggle: {
        method: 'PATCH',
        path: '/todos/:id/toggle',
    },
    update: {
        method: 'PUT',
        path: '/todos/:id',
    },
    delete: {
        method: 'DELETE',
        path: '/todos/:id',
    },
};
