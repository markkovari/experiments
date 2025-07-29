import { PostgreSqlContainer, StartedPostgreSqlContainer } from "@testcontainers/postgresql";
import { PrismaClient } from "./generated/prisma/client";
import { test, it, expect, describe, } from "vitest";
import path from "node:path"
import { exec as execCallback } from "node:child_process";
import { promisify } from "node:util";

const exec = promisify(execCallback);

interface CustomTestContext {
    prisma: PrismaClient;
    databaseContainer: StartedPostgreSqlContainer;
    databaseUrl: string;
}

const testBed = test.extend<CustomTestContext>({
    databaseContainer: async ({ }, use) => {
        console.log("Starting PostgreSQL container...");
        const container = await new PostgreSqlContainer("postgres:alpine")
            .withDatabase('postgres')
            .withUsername('postgres')
            .withPassword('postgres')
            .start();

        console.log(`PostgreSQL container started on port ${container.getMappedPort(5432)}`);

        await use(container);

        console.log("Stopping PostgreSQL container...");
        await container.stop();
        console.log("PostgreSQL container stopped.");
    },
    prisma: async ({ databaseUrl }, use) => {
        try {
            // Path to your prisma schema file. Adjust as needed.
            const schemaPath = path.resolve(__dirname, './prisma/schema.prisma'); // Assuming prisma folder is sibling to setupTests.ts

            await exec(`npx prisma migrate deploy --schema="${schemaPath}"`, {
                env: {
                    ...process.env,
                    DATABASE_URL: databaseUrl,
                },
            });
            await exec(`npx prisma db seed --schema="${schemaPath}"`, {
                env: {
                    ...process.env,
                    DATABASE_URL: databaseUrl,
                },
            });
            console.log("Prisma migrations applied successfully.");
        } catch (error) {
            console.error("Failed to apply Prisma migrations:", error);
            // Re-throw to fail the test if migrations don't apply
            throw error;
        }

        // 2. Initialize and connect Prisma Client
        const prisma = new PrismaClient({
            datasourceUrl: databaseUrl,
        });

        console.log("Connecting Prisma client...");
        await prisma.$connect();
        console.log("Prisma client connected.");

        // 3. Provide Prisma Client to the test
        await use(prisma);

        // 4. Disconnect Prisma Client after the test (and container stop)
        console.log("Disconnecting Prisma client...");
        await prisma.$disconnect();
        console.log("Prisma client disconnected.");
    },
    databaseUrl: async ({ databaseContainer }, use) => {
        use(databaseContainer.getConnectionUri())
    }
});


export { testBed as test, it, expect, describe, }