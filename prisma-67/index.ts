import express from "express";
import type { Request, Response } from "express";
import { Pool } from "pg";
import { PrismaPg } from "@prisma/adapter-pg";
import { PrismaClient } from "./generated/prisma/client";
import { randomUUID } from "node:crypto";

const app = express();

const port = process.env.PORT || 8080;

const connectionString =
	process.env.DATABASE_URL || "postgresql://localhost:5432/mydb";

const pool = new Pool({
	connectionString
});

const adapter = new PrismaPg(pool);

const client = new PrismaClient({
	adapter,
});

app.get("/add", async (req: Request, res: Response) => {
    const newUser = await client.user.create({data :{ email: randomUUID() }});
    return res.json({ newUser });
});

app.get("/", async (req: Request, res: Response) => {
	const users = await client.user.findMany();
	return res.json({ users });
});


app.listen(port, () => {
	console.log(`Server is running on port ${port}`);
});
