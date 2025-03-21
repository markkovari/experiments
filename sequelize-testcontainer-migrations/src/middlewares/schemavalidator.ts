import type { NextFunction, Request, Response } from "express";
import { ZodError, type ZodSchema } from "zod";

const validateBody =
  (schema: ZodSchema) => (req: Request, res: Response, next: NextFunction) => {
    try {
      schema.parse(req.body);
      next();
    } catch (error) {
      if (error instanceof ZodError) {
        res.status(400).json(error.errors);
        return;
      }
      res.status(400).json({ error: "unkown error" });
      return;
    }
  };

const validateParams =
  (schema: ZodSchema) => (req: Request, res: Response, next: NextFunction) => {
    try {
      schema.parse(req.params);
      next();
    } catch (error) {
      if (error instanceof ZodError) {
        res.status(400).json(error.errors);
        return;
      }
      res.status(400).json({ error: "unkown error" });
      return;
    }
  };

export { validateBody, validateParams };
