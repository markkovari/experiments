import express from "express";
import type { Request, Response } from "express";
import { Pool } from "pg";
import { PrismaPg } from "@prisma/adapter-pg";
import { PrismaClient } from "./generated/prisma/client";

const app = express();

const port = process.env.PORT || 3000;

const connectionString = process.env.DATABASE_URL || "postgresql://localhost:5432/mydb";

const pool = new Pool({
    connectionString,
});

const adapter = new PrismaPg(pool);

const client = new PrismaClient({
    adapter,
});

app.get("/", async (req: Request, res:Response) => {
    const users = await client.user.findMany();
    return res.json({ users });
});

app.listen(port, () => {
    console.log(`Server is running on port ${port}`);
});