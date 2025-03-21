import {
  PostgreSqlContainer,
  type StartedPostgreSqlContainer,
} from "@testcontainers/postgresql";
import { afterAll, beforeAll, beforeEach, describe, expect, it } from "vitest";

import type { Model, ModelCtor, Sequelize } from "sequelize";
import { getSequelize } from "../src/database";

let container: PostgreSqlContainer;
let startedContainer: StartedPostgreSqlContainer;
let sequelize: Sequelize;
// biome-ignore lint/suspicious/noExplicitAny: <explanation>
let UserModel: ModelCtor<Model<any, any>>;

beforeAll(async () => {
  container = new PostgreSqlContainer("postgres:16-alpine").withExposedPorts(
    5432,
  );
  startedContainer = await container.start();
  const { client, User } = await getSequelize({
    database: startedContainer.getDatabase(),
    dialect: "postgres",
    host: startedContainer.getHost(),
    password: startedContainer.getPassword(),
    port: startedContainer.getPort(),
    username: startedContainer.getPassword(),
  });
  UserModel = User;
  sequelize = client;
});

beforeEach(async () => {
  // option one
  // await UserModel.destroy({ force: true, cascade: true, truncate: true });
  await sequelize.truncate({ force: true, truncate: true, cascade: true });
});

afterAll(async () => {
  await sequelize.close();
  await startedContainer.stop();
});

describe("database", () => {
  describe("functionaly", () => {
    it("should be able to store users", async () => {
      const savedUser = await UserModel.create({
        firstName: "Mark",
        lastName: "Kovari",
      });
      expect(savedUser).not.toBeNull();
    });

    it("should be able to store multiple users", async () => {
      const savedUser = await UserModel.create({
        firstName: "Mark",
        lastName: "Kovari",
      });
      const savedUser2 = await UserModel.create({
        firstName: "Mark",
        lastName: "Kovari",
      });
      const userAmount = await UserModel.count();
      expect(userAmount).toBe(2);
    });
  });
});
