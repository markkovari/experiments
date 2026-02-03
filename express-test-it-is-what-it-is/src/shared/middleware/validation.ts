import type { NextFunction, Request, Response } from 'express';
import { type ZodType, ZodError, z } from 'zod';
import type { TypedRequest } from '../types/express-helpers';

/**
 * Validation error for clearer error messages
 */
export class ValidationError extends Error {
  constructor(
    public errors: Array<{ field: string; message: string }>,
  ) {
    super('Validation failed');
    this.name = 'ValidationError';
  }
}

/**
 * Type-safe validation middleware using Zod schemas
 *
 * Validates request body, params, or query against a Zod schema
 * On success: parsed data replaces req.body/params/query (with type coercion)
 * On failure: throws ValidationError with detailed field errors
 *
 * Usage:
 * router.post('/users',
 *   validate({ body: createUserSchema }),
 *   async (req: ValidatedRequest<typeof createUserSchema>, res) => {
 *     // req.body is fully typed!
 *   }
 * );
 */
export const validate = <
  BodySchema extends ZodType = ZodType,
  ParamsSchema extends ZodType = ZodType,
  QuerySchema extends ZodType = ZodType,
>(schemas: {
  body?: BodySchema;
  params?: ParamsSchema;
  query?: QuerySchema;
}) => {
  return async (req: Request, _res: Response, next: NextFunction) => {
    try {
      // Validate body
      if (schemas.body) {
        req.body = await schemas.body.parseAsync(req.body);
      }

      // Validate params
      if (schemas.params) {
        req.params = await schemas.params.parseAsync(req.params) as any;
      }

      // Validate query
      if (schemas.query) {
        req.query = await schemas.query.parseAsync(req.query) as any;
      }

      next();
    } catch (error) {
      if (error instanceof ZodError) {
        const validationErrors = error.issues.map((err: any) => ({
          field: err.path.join('.'),
          message: err.message,
        }));
        next(new ValidationError(validationErrors));
      } else {
        next(error);
      }
    }
  };
};

/**
 * Type helper for validated requests
 *
 * Use this to type your handler after validation middleware:
 *
 * const handler = async (
 *   req: ValidatedRequest<typeof createUserSchema, typeof userIdParamSchema>,
 *   res: Response
 * ) => {
 *   req.body // typed as CreateUserBody
 *   req.params // typed as UserIdParams
 * }
 */
export type ValidatedRequest<
  BodySchema extends ZodType = ZodType,
  ParamsSchema extends ZodType = ZodType,
  QuerySchema extends ZodType = ZodType,
> = TypedRequest<
  BodySchema extends ZodType ? z.infer<BodySchema> : unknown,
  ParamsSchema extends ZodType ? z.infer<ParamsSchema> : unknown,
  QuerySchema extends ZodType ? z.infer<QuerySchema> : unknown
>;
