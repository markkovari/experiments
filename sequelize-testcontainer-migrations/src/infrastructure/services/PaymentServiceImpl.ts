import type { Payment } from "../database/models/Payment";
import { User } from "../database/models/User";
import type { PaymentService } from "./PaymentService";

const paymentServiceImpl: PaymentService = {
  executeTransaction: async (p: Payment) => {
    const from = await User.findByPk(p.fromId);
    const to = await User.findByPk(p.toId);
    if (!from) return;
    if (!to) return;
    await User.update(
      { amount: from.amount - p.amount },
      { where: { id: from.id } },
    );
    await User.update(
      { amount: to.amount + p.amount },
      { where: { id: to.id } },
    );
  },
};

export { paymentServiceImpl };
