import { type Dialect, Sequelize } from "sequelize";
import {
  Payment,
  definePaymentModel,
} from "./infrastructure/database/models/Payment";
import { User, defineUserModel } from "./infrastructure/database/models/User";

type GetSequelizeArgs = {
  database: string;
  username: string;
  password: string;
  host: string;
  port: number;
  dialect: Dialect;
};
export async function getSequelize({
  database,
  username,
  password,
  host,
  port,
  dialect,
}: GetSequelizeArgs) {
  // Create the client
  const client = new Sequelize(database, username, password, {
    host,
    dialect,
    port,
    logging: false,
  });
  // Define a model
  defineUserModel(client);
  definePaymentModel(client);

  // Make sure the table exists
  await User.sync({ alter: false });
  await Payment.sync({ alter: false });
  // Return the client and model
  return { client, User, Payment };
}
