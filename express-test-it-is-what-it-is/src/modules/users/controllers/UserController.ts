import { Request, Response } from 'express';
import { UserService } from '../services/UserService';

export class UserController {
  constructor(private userService: UserService) {}

  getAll = async (req: Request, res: Response): Promise<void> => {
    try {
      const users = await this.userService.getAllUsers();
      res.json(users.map((user) => user.toDTO()));
    } catch (error) {
      res.status(500).json({ error: 'Internal server error' });
    }
  };

  getById = async (req: Request, res: Response): Promise<void> => {
    try {
      const { id } = req.params;
      const user = await this.userService.getUserById(id);

      if (!user) {
        res.status(404).json({ error: 'User not found' });
        return;
      }

      res.json(user.toDTO());
    } catch (error) {
      res.status(500).json({ error: 'Internal server error' });
    }
  };

  create = async (req: Request, res: Response): Promise<void> => {
    try {
      const { email, name, password } = req.body;

      if (!email || !name || !password) {
        res.status(400).json({ error: 'Missing required fields' });
        return;
      }

      const user = await this.userService.createUser({ email, name, password });
      res.status(201).json(user.toDTO());
    } catch (error) {
      if (error instanceof Error && error.message === 'User with this email already exists') {
        res.status(409).json({ error: error.message });
        return;
      }
      res.status(500).json({ error: 'Internal server error' });
    }
  };

  update = async (req: Request, res: Response): Promise<void> => {
    try {
      const { id } = req.params;
      const { name, email } = req.body;

      const user = await this.userService.updateUser(id, { name, email });
      res.json(user.toDTO());
    } catch (error) {
      if (error instanceof Error && error.message === 'User not found') {
        res.status(404).json({ error: error.message });
        return;
      }
      if (error instanceof Error && error.message === 'Email already in use') {
        res.status(409).json({ error: error.message });
        return;
      }
      res.status(500).json({ error: 'Internal server error' });
    }
  };

  delete = async (req: Request, res: Response): Promise<void> => {
    try {
      const { id } = req.params;
      await this.userService.deleteUser(id);
      res.status(204).send();
    } catch (error) {
      if (error instanceof Error && error.message === 'User not found') {
        res.status(404).json({ error: error.message });
        return;
      }
      res.status(500).json({ error: 'Internal server error' });
    }
  };
}
