import { defineConfig } from 'vitest/config'

export default defineConfig({
    test: {
        setupFiles: ["setupTests.ts"],
        fileParallelism: false,
        poolOptions: {
            threads: {
                minThreads: 1,
                maxThreads: 20,
                isolate: false,
                singleThread: false
            }
        },
        sequence: {
            concurrent: true
        },
        isolate: false,
    },

})