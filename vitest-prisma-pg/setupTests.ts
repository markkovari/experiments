import { PrismaClient } from "./generated/prisma/client";
import { test, it, expect, describe, } from "vitest";



const testBed = test.extend<{ prisma: PrismaClient }>({
    prisma: async ({ }, use) => {
        const prisma = new PrismaClient({ datasourceUrl: "" })
        await prisma.$connect()
        await use(prisma);
        await prisma.$disconnect()
    }
});


export { testBed as test, it, expect, describe, }