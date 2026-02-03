import { IUserRepository } from '../repository/IUserRepository';
import { User } from '../domain/User';
import { NotFoundError, ConflictError } from '../../../shared/middleware/error-handler';

export class UserService {
  constructor(private userRepository: IUserRepository) {}

  async getUserById(id: string): Promise<User> {
    const user = await this.userRepository.findById(id);
    if (!user) throw new NotFoundError('User');
    return user;
  }

  async getUserByEmail(email: string): Promise<User | null> {
    return this.userRepository.findByEmail(email);
  }

  async getAllUsers(): Promise<User[]> {
    return this.userRepository.findAll();
  }

  async createUser(data: { email: string; name: string; password: string }): Promise<User> {
    const existingUser = await this.userRepository.findByEmail(data.email);
    if (existingUser) {
      throw new ConflictError('User with this email already exists');
    }

    return this.userRepository.create(data);
  }

  async updateUser(id: string, data: { name?: string; email?: string }): Promise<User> {
    const user = await this.userRepository.findById(id);
    if (!user) throw new NotFoundError('User');

    if (data.email && data.email !== user.email) {
      const existingUser = await this.userRepository.findByEmail(data.email);
      if (existingUser) {
        throw new ConflictError('Email already in use');
      }
    }

    return this.userRepository.update(id, data);
  }

  async deleteUser(id: string): Promise<void> {
    const user = await this.userRepository.findById(id);
    if (!user) throw new NotFoundError('User');

    await this.userRepository.delete(id);
  }
}
