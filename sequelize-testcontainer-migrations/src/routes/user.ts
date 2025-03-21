import { NextFunction, type Request, Response, Router } from "express";
import { z } from "zod";

import { UserRepo } from "../infrastructure/repositories/UserRepoImpl";
import { validateParams } from "../middlewares/schemavalidator";

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
  res.status(201).json(byId);
});

export { userRouter };
