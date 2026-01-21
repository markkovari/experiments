import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    testTimeout: 120000,
    hookTimeout: 120000,
    globalSetup: ['./tests/globalSetup.ts'],
    setupFiles: ['./tests/setup.ts'],
    include: ['tests/stress/**/*.test.ts'],
    fileParallelism: true,
    pool: 'forks',
    poolOptions: {
      forks: {
        singleFork: false,
        isolate: true,
        maxForks: 4,
        minForks: 1,
      },
    },
    sequence: {
      shuffle: false,
    },
    retry: 2,
  },
});
