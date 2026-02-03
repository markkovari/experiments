import { User } from '../domain/User';

export interface IUserRepository {
  findById(id: string): Promise<User | null>;
  findByEmail(email: string): Promise<User | null>;
  findAll(): Promise<User[]>;
  create(data: { email: string; name: string; password: string }): Promise<User>;
  update(id: string, data: { name?: string; email?: string }): Promise<User>;
  delete(id: string): Promise<void>;
}
