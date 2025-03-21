import { User } from "../database/models/User";
import type { IUserRepo } from "./UserRepo";

const UserRepo = (): IUserRepo => ({
  create: async (details) => await User.create(details),
  getAll: async () => await User.findAll(),
  delete: async (id: number) => await User.destroy({ where: { id } }),
  getById: async (id: number) => await User.findByPk(id),
  update: async (id: number, details) =>
    await User.update({ firstName: details.firstName }, { where: { id } }),
});

export { UserRepo };
