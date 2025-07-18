import type { Config } from "drizzle-kit";

import { env } from "@magic/env";
console.log({ url: env.DATABASE_URL })

export default {
	schema: "schema.ts",
	dialect: "postgresql",
	dbCredentials: {
		url: env.DATABASE_URL,
	},
	tablesFilter: ["meta_*"],
} satisfies Config;
