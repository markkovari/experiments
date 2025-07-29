import { PrismaClient } from "./generated/prisma/client";
import { PostgreSqlContainer, StartedPostgreSqlContainer } from "@testcontainers/postgresql";
import { beforeAll, afterAll, } from "vitest";

import path from "node:path"
import { exec as execCallback } from "node:child_process";
import { promisify } from "node:util";
import fs from "node:fs"


const exec = promisify(execCallback);

export const TEMPLATE_SCHEMA_NAME = "test_template_schema"; // Define your template schema name
const CLONE_SCHEMA_SQL_PATH = path.resolve(__dirname, './functions/clone_schema.sql'); // Path to your clone_schema.sql


let globalPostgresContainer: StartedPostgreSqlContainer;


beforeAll(async () => {
    console.log("Starting a SINGLE global PostgreSQL container for all tests...");
    globalPostgresContainer = await new PostgreSqlContainer("postgres:alpine")
        .withDatabase('maintestdb')
        .withUsername('testuser')
        .withPassword('testpassword')
        .start();
    console.log(`Global PostgreSQL container started on port ${globalPostgresContainer.getMappedPort(5432)}.`);

    const databaseUrl = globalPostgresContainer.getConnectionUri();
    const tempPrismaClient = new PrismaClient({ datasourceUrl: databaseUrl, });
    await tempPrismaClient.$connect();
    try {
        // 1. Install the clone_schema function
        console.log(`Installing clone_schema function...`);
        const cloneSchemaSql = fs.readFileSync(CLONE_SCHEMA_SQL_PATH, 'utf8');
        await tempPrismaClient.$executeRawUnsafe(cloneSchemaSql);
        console.log(`clone_schema function installed.`);

        // 2. Create the template schema
        console.log(`Creating template schema: ${TEMPLATE_SCHEMA_NAME}`);
        await tempPrismaClient.$executeRawUnsafe(`CREATE SCHEMA IF NOT EXISTS "${TEMPLATE_SCHEMA_NAME}";`);

        // 3. Apply Prisma migrations to the template schema once
        console.log(`Applying Prisma migrations to template schema: ${TEMPLATE_SCHEMA_NAME}`);
        const templateSchemaDatabaseUrl = `${databaseUrl}?schema=${TEMPLATE_SCHEMA_NAME}`;
        const schemaPath = path.resolve(__dirname, './prisma/schema.prisma'); // Adjust path

        await exec(`npx prisma migrate deploy --schema="${schemaPath}"`, {
            env: {
                ...process.env,
                DATABASE_URL: templateSchemaDatabaseUrl,
            },
        });
        console.log("Prisma migrations applied to template schema successfully.");


    } catch (error) {
        console.error("Failed to set up global template schema:", error);
        throw error; // Fail fast if global setup fails
    } finally {
        await tempPrismaClient.$disconnect();
    }

}, 120000);

afterAll(async () => {
    console.log("Stopping the global PostgreSQL container...");
    if (globalPostgresContainer) {
        await globalPostgresContainer.stop();
        console.log("Global PostgreSQL container stopped.");
    }
});


export { globalPostgresContainer }