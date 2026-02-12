/**
 * API endpoint definitions
 * Provides a single source of truth for API routes
 */
export declare const TodoEndpoints: {
    readonly list: {
        readonly method: "GET";
        readonly path: "/todos";
    };
    readonly add: {
        readonly method: "POST";
        readonly path: "/todos";
    };
    readonly toggle: {
        readonly method: "PATCH";
        readonly path: "/todos/:id/toggle";
    };
    readonly update: {
        readonly method: "PUT";
        readonly path: "/todos/:id";
    };
    readonly delete: {
        readonly method: "DELETE";
        readonly path: "/todos/:id";
    };
};
