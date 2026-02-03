import { PostgreSqlContainer } from '@testcontainers/postgresql';
import { PrismaClient } from '@prisma/client';
import { PrismaPg } from '@prisma/adapter-pg';
import { Pool } from 'pg';
import { execSync } from 'child_process';
import type { StartedPostgreSqlContainer } from '@testcontainers/postgresql';

let container: StartedPostgreSqlContainer;
let prisma: PrismaClient;
let pool: Pool;

export async function setupTestDatabase() {
  // Start PostgreSQL container
  container = await new PostgreSqlContainer('postgres:16-alpine')
    .withDatabase('testdb')
    .withUsername('testuser')
    .withPassword('testpass')
    .start();

  const databaseUrl = container.getConnectionUri();
  process.env.DATABASE_URL = databaseUrl;

  // Run migrations
  execSync('pnpm prisma migrate deploy', {
    env: { ...process.env, DATABASE_URL: databaseUrl },
  });

  // Initialize Prisma Client with pg adapter
  pool = new Pool({ connectionString: databaseUrl });
  const adapter = new PrismaPg(pool);
  prisma = new PrismaClient({ adapter });

  return { prisma, databaseUrl };
}

export async function teardownTestDatabase() {
  if (prisma) {
    await prisma.$disconnect();
  }
  if (pool) {
    await pool.end();
  }
  if (container) {
    await container.stop();
  }
}

export function getPrismaClient(): PrismaClient {
  if (!prisma) {
    throw new Error('Prisma client not initialized. Call setupTestDatabase first.');
  }
  return prisma;
}

export async function cleanDatabase() {
  if (!prisma) return;

  // Clean all tables in reverse order of dependencies
  await prisma.post.deleteMany();
  await prisma.user.deleteMany();
}
