import {
  type Association,
  DataTypes,
  Model,
  type Optional,
  type Sequelize,
} from "sequelize";
import { Payment } from "./Payment";

type UserAttributes = {
  id: number;
  firstName: string;
  lastName: string;
  email: string;
  amount?: number;

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
  public email!: string;
  public amount!: number;

  public createdAt!: Date;
  public updatedAt!: Date;
  declare static associations: {
    payments: Association<User, Payment>;
  };

  static associate() {
    User.hasMany(Payment, { foreignKey: "fromId", as: "payments" });
    User.hasMany(Payment, { foreignKey: "toId", as: "payments" });
  }
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
      email: {
        type: DataTypes.STRING,
        unique: true,
      },
      amount: {
        type: DataTypes.INTEGER,
        defaultValue: 0,
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
