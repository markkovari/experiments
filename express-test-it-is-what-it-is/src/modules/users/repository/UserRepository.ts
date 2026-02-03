import { PrismaClient } from '@prisma/client';
import { User } from '../domain/User';
import { IUserRepository } from './IUserRepository';

export class UserRepository implements IUserRepository {
  constructor(private prisma: PrismaClient) {}

  async findById(id: string): Promise<User | null> {
    const user = await this.prisma.user.findUnique({
      where: { id },
    });

    return user ? this.toDomain(user) : null;
  }

  async findByEmail(email: string): Promise<User | null> {
    const user = await this.prisma.user.findUnique({
      where: { email },
    });

    return user ? this.toDomain(user) : null;
  }

  async findAll(): Promise<User[]> {
    const users = await this.prisma.user.findMany();
    return users.map((user) => this.toDomain(user));
  }

  async create(data: { email: string; name: string; password: string }): Promise<User> {
    const user = await this.prisma.user.create({
      data,
    });

    return this.toDomain(user);
  }

  async update(id: string, data: { name?: string; email?: string }): Promise<User> {
    const user = await this.prisma.user.update({
      where: { id },
      data,
    });

    return this.toDomain(user);
  }

  async delete(id: string): Promise<void> {
    await this.prisma.user.delete({
      where: { id },
    });
  }

  private toDomain(user: any): User {
    return User.create({
      id: user.id,
      email: user.email,
      name: user.name,
      password: user.password,
      createdAt: user.createdAt,
      updatedAt: user.updatedAt,
    });
  }
}
