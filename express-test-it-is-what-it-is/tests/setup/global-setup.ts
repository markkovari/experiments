import { setupTestDatabase, teardownTestDatabase } from './testcontainers';

export async function setup() {
  // This runs once before all tests
  const testType = process.env.TEST_TYPE;

  // Only setup testcontainers for integration and e2e tests
  if (testType === 'integration' || testType === 'e2e') {
    await setupTestDatabase();
  }
}

export async function teardown() {
  const testType = process.env.TEST_TYPE;

  if (testType === 'integration' || testType === 'e2e') {
    await teardownTestDatabase();
  }
}
