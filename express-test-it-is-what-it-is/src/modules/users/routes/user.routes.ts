import { Router } from 'express';
import { UserController } from '../controllers/UserController';
import { validate } from '../../../shared/middleware/validation';
import { asyncHandler } from '../../../shared/middleware/async-handler';
import { createUserSchema, updateUserSchema, userIdParamSchema } from '../schemas/user.schemas';

export function createUserRoutes(userController: UserController): Router {
  const router = Router();

  // GET /users - No validation needed
  router.get('/', asyncHandler(userController.getAll));

  // GET /users/:id - Validate ID param
  router.get('/:id', validate({ params: userIdParamSchema }), asyncHandler(userController.getById));

  // POST /users - Validate body
  router.post('/', validate({ body: createUserSchema }), asyncHandler(userController.create));

  // PUT /users/:id - Validate both params and body
  router.put(
    '/:id',
    validate({ params: userIdParamSchema, body: updateUserSchema }),
    asyncHandler(userController.update),
  );

  // DELETE /users/:id - Validate ID param
  router.delete('/:id', validate({ params: userIdParamSchema }), asyncHandler(userController.delete));

  return router;
}
