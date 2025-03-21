import { Router } from "express";
import { userRouter } from "./user";

const mainRouter = Router();

mainRouter.use("/api/v1/users", userRouter);

export { mainRouter };
