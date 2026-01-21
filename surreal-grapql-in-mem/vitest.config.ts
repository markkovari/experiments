import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    testTimeout: 60000,
    hookTimeout: 120000,
    setupFiles: ['./tests/setup.ts'],
    include: ['tests/**/*.test.ts'],
    fileParallelism: true,
    pool: 'forks',
    poolOptions: {
      forks: {
        singleFork: false,
        isolate: true,
      },
    },
  },
});
