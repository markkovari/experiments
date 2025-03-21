import type { User, UserCreationAttributes } from "../database/models/User";

export type IUserRepo = {
  getAll: () => Promise<User[]>;
  getById: (id: number) => Promise<User | null>;
  create(details: UserCreationAttributes): Promise<User>;
  delete(id: number): Promise<number>;
  update(
    id: number,
    details: Omit<User, "id" | "createdAt" | "updatedAt">,
  ): Promise<[affectedCount: number]>;
};
