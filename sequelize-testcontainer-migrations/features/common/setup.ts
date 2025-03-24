import { AfterAll, BeforeAll } from "@cucumber/cucumber";
import {
  PostgreSqlContainer,
  type StartedPostgreSqlContainer,
} from "@testcontainers/postgresql";
import type { Sequelize } from "sequelize";
import { getSequelize } from "../../src/database";

let container: PostgreSqlContainer;
let startedContainer: StartedPostgreSqlContainer;
let sequelize: Sequelize;

BeforeAll(async () => {
  container = new PostgreSqlContainer("postgres:16-alpine").withExposedPorts(
    5432,
  );
  startedContainer = await container.start();
  const { client } = await getSequelize({
    database: startedContainer.getDatabase(),
    dialect: "postgres",
    host: startedContainer.getHost(),
    password: startedContainer.getPassword(),
    port: startedContainer.getPort(),
    username: startedContainer.getPassword(),
  });
  sequelize = client;
});

// BeforeStep(async () => {
// 	await sequelize.truncate({ force: true, truncate: true, cascade: true });
// });

// afterEach(async () => {
// 	await sequelize.truncate({ force: true, truncate: true, cascade: true });
// });

AfterAll(async () => {
  await sequelize.close();
  await startedContainer.stop();
});

export { container, startedContainer, sequelize };
