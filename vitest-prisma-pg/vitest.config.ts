import { defineConfig } from 'vitest/config'

export default defineConfig({
    test: {
        pool: 'threads',
        poolOptions: {
            threads: {
                maxThreads: 20,
                minThreads: 4,
                isolate: true
            },
        },
        sequence: {
            concurrent: true,
        },
        testTimeout: 60000,
        hookTimeout: 120000,
    },
})