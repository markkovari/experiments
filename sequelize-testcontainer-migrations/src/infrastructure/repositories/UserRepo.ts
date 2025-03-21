import { z } from "zod";
import type { User, UserCreationAttributes } from "../database/models/User";

//TODO: should be in a few layers above
export const userUpdateBodySchema = z.object({
  firstName: z
    .string()
    .min(1, "firstName must be at least 1 character long")
    .optional(),
  lastName: z
    .string()
    .min(1, "firstName must be at least 1 character long")
    .optional(),
});

export type UserUpdateBody = z.infer<typeof userUpdateBodySchema>;

export type IUserRepo = {
  getAll: () => Promise<User[]>;
  getById: (id: number) => Promise<User | null>;
  create(details: UserCreationAttributes): Promise<User>;
  delete(id: number): Promise<number>;
  update(id: number, details: UserUpdateBody): Promise<[affectedCount: number]>;
};
