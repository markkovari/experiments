import { Router } from "express";
import { UserRepo } from "../infrastructure/repositories/UserRepoImpl";

const userRouter = Router();

userRouter.get("/", async (req, res) => {
  const userRepo = UserRepo();
  res.json(await userRepo.getAll());
});

export { userRouter };
