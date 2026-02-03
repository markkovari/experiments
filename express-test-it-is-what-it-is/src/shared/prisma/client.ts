import { PrismaClient } from '@prisma/client';
import { PrismaPg } from '@prisma/adapter-pg';
import { Pool } from 'pg';

const globalForPrisma = global as unknown as { prisma?: PrismaClient; pool?: Pool };

function createPrismaClient(): PrismaClient {
  // Prisma 7 requires adapter or accelerateUrl
  // Use pg adapter with DATABASE_URL from environment
  if (process.env.DATABASE_URL) {
    const pool = globalForPrisma.pool || new Pool({ connectionString: process.env.DATABASE_URL });
    if (!globalForPrisma.pool) globalForPrisma.pool = pool;

    const adapter = new PrismaPg(pool);
    return new PrismaClient({
      adapter,
      log: ['query', 'error', 'warn'],
    });
  }

  // Fallback for when DATABASE_URL is not set (shouldn't happen in normal usage)
  throw new Error('DATABASE_URL environment variable is required. Make sure to set it in .env or environment.');
}

// Lazy initialization - create client on first access
// This allows DATABASE_URL to be set after module load (important for tests)
function getPrismaClient(): PrismaClient {
  if (!globalForPrisma.prisma) {
    globalForPrisma.prisma = createPrismaClient();
  }
  return globalForPrisma.prisma;
}

// Export a Proxy that creates the client on first access
export const prisma = new Proxy({} as PrismaClient, {
  get(target, prop) {
    const client = getPrismaClient();
    const value = client[prop as keyof PrismaClient];
    return typeof value === 'function' ? value.bind(client) : value;
  },
});
