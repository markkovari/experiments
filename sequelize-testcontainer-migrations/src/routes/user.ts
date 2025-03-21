import { Router } from "express";

import { z } from "zod";
import {
  type UserUpdateBody,
  userUpdateBodySchema,
} from "../infrastructure/repositories/UserRepo";
import { UserRepo } from "../infrastructure/repositories/UserRepoImpl";
import { validateBody } from "../middlewares/schemavalidator";

const userRouter = Router();

userRouter.get("/", async (req, res) => {
  const userRepo = UserRepo();
  res.json(await userRepo.getAll());
});

userRouter.post("/", async (req, res) => {
  const userRepo = UserRepo();
  res.status(201).json(await userRepo.create(req.body));
});

userRouter.get("/:id", async (req, res) => {
  const { id } = req.params;
  const asInt = Number.parseInt(id, 10);
  if (Number.isNaN(asInt)) {
    res.status(400).json({ error: "Id is not a number" });
    return;
  }
  const userRepo = UserRepo();
  const byId = await userRepo.getById(asInt);
  if (!byId) {
    res.status(404).send();
    return;
  }
  res.status(200).json(byId);
});

userRouter.delete("/:id", async (req, res) => {
  const { id } = req.params;
  const asInt = Number.parseInt(id, 10);
  if (Number.isNaN(asInt)) {
    res.status(400).json({ error: "Id is not a number" });
    return;
  }
  const userRepo = UserRepo();
  const byId = await userRepo.delete(asInt);
  if (!byId) {
    res.status(404).send();
    return;
  }
  res.status(200).json(byId);
});

userRouter.post(
  "/:id",
  validateBody(userUpdateBodySchema),
  async (req, res) => {
    const { id } = req.params;
    const details: UserUpdateBody = req.body;
    const asInt = Number.parseInt(id, 10);
    if (Number.isNaN(asInt)) {
      res.status(400).json({ error: "Id is not a number" });
      return;
    }
    const userRepo = UserRepo();
    const byId = await userRepo.update(asInt, details);
    if (!byId) {
      res.status(404).send();
      return;
    }
    res.status(200).json(byId);
  },
);

export { userRouter };
