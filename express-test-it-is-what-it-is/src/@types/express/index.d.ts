/**
 * Express type extensions
 *
 * This file intentionally kept minimal.
 * Use TypedRequest and AuthenticatedRequest from shared/types/express-helpers.ts instead
 * of globally extending the Express Request interface.
 *
 * This approach ensures TypeScript knows when req.user is available (only in AuthenticatedRequest)
 * vs when it's not (TypedRequest), preventing false assumptions.
 */

export {};
