import { Request, Response, NextFunction } from 'express';

/**
 * User type for authenticated requests
 */
export interface AuthUser {
  id: string;
  email: string;
  name: string;
}

/**
 * Typed request for routes that don't require authentication
 * Provides type-safe body, params, and query
 */
export interface TypedRequest<
  Body = any,
  Params = Record<string, string>,
  Query = Record<string, string>,
> extends Request {
  body: Body;
  params: Params;
  query: Query;
}

/**
 * Typed request for routes that require authentication
 * Includes the authenticated user in addition to typed body, params, and query
 */
export interface AuthenticatedRequest<
  Body = any,
  Params = Record<string, string>,
  Query = Record<string, string>,
> extends TypedRequest<Body, Params, Query> {
  user: AuthUser;
}

/**
 * Handler type for routes that don't require authentication
 */
export type TypedRequestHandler<
  Body = any,
  Params = Record<string, string>,
  Query = Record<string, string>,
> = (req: TypedRequest<Body, Params, Query>, res: Response) => Promise<void> | void;

/**
 * Handler type for routes that require authentication
 * TypeScript will enforce that req.user is available
 */
export type AuthenticatedRequestHandler<
  Body = any,
  Params = Record<string, string>,
  Query = Record<string, string>,
> = (req: AuthenticatedRequest<Body, Params, Query>, res: Response) => Promise<void> | void;

/**
 * Middleware type
 */
export type TypedMiddleware = (req: Request, res: Response, next: NextFunction) => Promise<void> | void;
