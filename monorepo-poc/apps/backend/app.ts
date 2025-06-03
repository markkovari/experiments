import { calc } from "fibonacci";
import express, {} from "express";
import type { Application, Request, Response } from "express";
import { z } from "zod";

const app: Application = express();

const fibonacciRequestSchema = z.object({
	n: z
		.number({ message: "n is not a number" })
		.positive({ message: "n is not a positive number" }),
});

app.get("/calc/:n", (req: Request, res: Response) => {
	const validated = fibonacciRequestSchema.safeParse(req.params);
	if (!validated.success) {
		res.status(400).json({ error: validated.error });
		return;
	}
	const result = calc(validated.data.n);
	res.status(200).json({ result });
	return;
});

export { app };
