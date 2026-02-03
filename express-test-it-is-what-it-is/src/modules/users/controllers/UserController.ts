import { Request, Response } from 'express';
import { UserService } from '../services/UserService';
import { ValidatedRequest } from '../../../shared/middleware/validation';
import {
  createUserSchema,
  updateUserSchema,
  userIdParamSchema
} from '../schemas/user.schemas';

export class UserController {
  constructor(private userService: UserService) {}

  // No validation needed - no body/params
  getAll = async (_req: Request, res: Response): Promise<void> => {
    const users = await this.userService.getAllUsers();
    res.json(users.map((user) => user.toDTO()));
  };

  // Type-safe params after validation
  getById = async (
    req: ValidatedRequest<any, typeof userIdParamSchema>,
    res: Response
  ): Promise<void> => {
    const { id } = req.params; // TypeScript knows: { id: string } (valid UUID)
    const user = await this.userService.getUserById(id);
    res.json(user.toDTO());
  };

  // Type-safe body after validation
  create = async (
    req: ValidatedRequest<typeof createUserSchema>,
    res: Response
  ): Promise<void> => {
    const { email, name, password } = req.body; // TypeScript knows exact shape
    const user = await this.userService.createUser({ email, name, password });
    res.status(201).json(user.toDTO());
  };

  // Type-safe params + body after validation
  update = async (
    req: ValidatedRequest<typeof updateUserSchema, typeof userIdParamSchema>,
    res: Response
  ): Promise<void> => {
    const { id } = req.params; // Typed as { id: string }
    const { name, email } = req.body; // Typed as { name?: string, email?: string }

    const user = await this.userService.updateUser(id, { name, email });
    res.json(user.toDTO());
  };

  // Type-safe params after validation
  delete = async (
    req: ValidatedRequest<any, typeof userIdParamSchema>,
    res: Response
  ): Promise<void> => {
    const { id } = req.params;
    await this.userService.deleteUser(id);
    res.status(204).send();
  };
}
