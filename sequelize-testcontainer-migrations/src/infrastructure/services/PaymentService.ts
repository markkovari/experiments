import type { Payment } from "../database/models/Payment";

export type PaymentService = {
  executeTransaction(payment: Payment): Promise<void>;
};
