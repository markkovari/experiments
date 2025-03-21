import { DataTypes, Model, type Optional, type Sequelize } from "sequelize";

type UserAttributes = {
  id: number;
  firstName: string;
  lastName: string;

  createdAt?: Date;
  updatedAt?: Date;
};

export interface UserCreationAttributes
  extends Optional<UserAttributes, "id"> {}

class User
  extends Model<UserAttributes, UserCreationAttributes>
  implements UserAttributes
{
  public id!: number;
  public firstName!: string;
  public lastName!: string;
  public createdAt!: Date;
  public updatedAt!: Date;
}

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
      timestamps: true,
      tableName: "users",
    },
  );
};

export { defineUserModel, User };
