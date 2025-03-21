import { DataTypes, Model, type Sequelize } from "sequelize";

class User extends Model {}

const defineUserModel = (sequelize: Sequelize) => {
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
      sequelize,
      tableName: "users",
    },
  );
};

export { defineUserModel, User };
