import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		silent: false,
		printConsoleTrace: true,
		include: ["tests/**.test.ts"],
		setupFiles: ["tests/setup.ts"],
		globals: true,
		testTimeout: 60000,
		hookTimeout: 60000,
	},
});
