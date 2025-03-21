import { DataTypes, Model, type Sequelize } from "sequelize";
import { User } from "./User";

class Payment extends Model {}

const definePaymentModel = (sequelize: Sequelize) => {
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
      to: {
        type: DataTypes.INTEGER,
        allowNull: false,
        references: {
          key: "id",
          model: User,
        },
      },
    },
    {
      sequelize,
      tableName: "Payment",
    },
  );
};

export { definePaymentModel, Payment };
