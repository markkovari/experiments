import {
  DataTypes,
  type HasOneGetAssociationMixin,
  Model,
  type Optional,
  type Sequelize,
} from "sequelize";
import { User } from "./User";

type PaymentAttributes = {
  id: number;
  amount: number;
  fromId: number;
  toId: number;
};

interface PaymentCreationAttributes extends Optional<PaymentAttributes, "id"> {}

class Payment
  extends Model<PaymentCreationAttributes, PaymentCreationAttributes>
  implements PaymentAttributes
{
  public id!: number;
  public amount!: number;
  public fromId!: number;
  public toId!: number;

  public getFromUser!: HasOneGetAssociationMixin<User>;
  public getToUser!: HasOneGetAssociationMixin<User>;

  public readonly createdAt!: Date;
  public readonly updatedAt!: Date;

  static associate(models: { User: typeof User }) {
    Payment.hasOne(models.User, { foreignKey: "fromId", as: "from" });
    Payment.hasOne(models.User, { foreignKey: "toId", as: "from" });
  }
}

const definePaymentModel = (sequelize: Sequelize) => {
  Payment.init(
    {
      amount: {
        type: DataTypes.INTEGER,
        allowNull: false,
      },
      fromId: {
        type: DataTypes.INTEGER,
        references: {
          model: User,
          key: "id",
        },
      },
      toId: {
        type: DataTypes.INTEGER,
        references: {
          model: User,
          key: "id",
        },
      },
    },
    {
      sequelize,
      timestamps: true,
      tableName: "Payment",
    },
  );
};

export { definePaymentModel, Payment };
