import { DataTypes, type Dialect, Model, Sequelize } from "sequelize";

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
  class User extends Model {}
  User.init(
    {
      id: {
        type: DataTypes.INTEGER,
        primaryKey: true,
        autoIncrement: true,
      },
      firstName: {
        type: DataTypes.STRING,
        allowNull: false,
      },
      lastName: {
        type: DataTypes.STRING,
      },
    },
    {
      sequelize: client,
      tableName: "users",
    },
  );

  class Payment extends Model {}
  Payment.init(
    {
      amount: {
        type: DataTypes.INTEGER,
        allowNull: false,
      },
      from: {
        type: DataTypes.INTEGER,
        allowNull: false,
        references: {
          key: "id",
          model: User,
        },
      },
    },
    {
      sequelize: client,
      tableName: "Payment",
    },
  );
  // Make sure the table exists
  await User.sync();
  await Payment.sync();
  // Return the client and model
  return { client, User, Payment };
}
