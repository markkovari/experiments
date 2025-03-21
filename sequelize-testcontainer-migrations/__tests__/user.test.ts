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
// biome-ignore lint/suspicious/noExplicitAny: <explanation>
let PaymentModel: ModelCtor<Model<any, any>>;

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
    UserModel = User;
    PaymentModel = Payment;
    sequelize = client;
  },
  3 * 60 * 1000,
); // testcontainers take some time to load initially - 3 minutes is given

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

    it("should be able to create transfers from one account to another", async () => {
      const oldG = await UserModel.create({
        firstName: "George",
        lastName: "Soros",
      });
      const me = await UserModel.create({
        firstName: "Mark",
        lastName: "Kovari",
      });

      const payment = await PaymentModel.create({
        amount: 10000000,
        //@ts-ignore
        from: oldG.dataValues.id,
        //@ts-ignore
        to: me.dataValues.id,
      });
      expect(payment.dataValues.from).toBe(oldG.dataValues.id);
      //@ts-ignore
      expect(payment.dataValues.to).toBe(me.dataValues.id);
    });
  });
});
