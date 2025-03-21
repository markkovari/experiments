import {
	PostgreSqlContainer,
	type StartedPostgreSqlContainer,
} from "@testcontainers/postgresql";
import type { Sequelize } from "sequelize";
import { afterAll, beforeAll, beforeEach } from "vitest";
import { getSequelize } from "./src/database";
import { definePaymentModel } from "./src/infrastructure/database/models/Payment";
import { defineUserModel } from "./src/infrastructure/database/models/User";

let container: PostgreSqlContainer;
let startedContainer: StartedPostgreSqlContainer;
let sequelize: Sequelize;

beforeAll(
	async () => {
		container = new PostgreSqlContainer("postgres:16-alpine").withExposedPorts(
			5432,
		);
		startedContainer = await container.start();
		const { client, User, Payment } = await getSequelize({
			database: startedContainer.getDatabase(),
			dialect: "postgres",
			host: startedContainer.getHost(),
			password: startedContainer.getPassword(),
			port: startedContainer.getPort(),
			username: startedContainer.getPassword(),
		});
		defineUserModel(client);
		definePaymentModel(client);
		sequelize = client;
	},
	3 * 60 * 1000,
);

beforeEach(async () => {
	await sequelize.truncate({ force: true, truncate: true, cascade: true });
});

afterAll(async () => {
	await sequelize.close();
	await startedContainer.stop();
});

export { container, startedContainer, sequelize };
