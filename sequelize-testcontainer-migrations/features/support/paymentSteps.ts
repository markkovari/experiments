import { equal, notEqual } from "node:assert";
import { Given, Then, When } from "@cucumber/cucumber";

import { Payment } from "../../src/infrastructure/database/models/Payment";
import { User } from "../../src/infrastructure/database/models/User";
import { paymentServiceImpl } from "../../src/infrastructure/services/PaymentServiceImpl";

const initials: { [key: string]: number } = {};
let payment: Payment;

Given("there are no users in the system", async () => {
  await User.truncate({ cascade: true, force: true });
});

Given(
  "a user with email: {string} and {int} as amount",
  async (email: string, amount: number) => {
    await User.create({
      email,
      firstName: "First",
      lastName: "Lastname",
      amount,
    });
    initials[email] = amount;
  },
);

When(
  "the user with email: {string} pays to the user with: {string} with {int}",
  async (emailFrom: string, emailTo: string, int: number) => {
    const from = await User.findOne({ where: { email: emailFrom } });
    if (!from) throw Error();
    const to = await User.findOne({ where: { email: emailTo } });
    if (!to) throw Error();
    payment = await Payment.create({
      amount: int,
      fromId: from.id,
      toId: to.id,
    });
  },
);

Then(
  "the payment is successfully registered from {string} to {string}",
  async (from: string, to: string) => {
    const fromUser = await User.findOne({ where: { email: from } });
    if (!fromUser) throw Error();
    const toUser = await User.findOne({ where: { email: to } });
    if (!toUser) throw Error();
    const storedPayment = await Payment.findOne({
      where: {
        fromId: fromUser.id,
        toId: toUser.id,
      },
    });
    notEqual(storedPayment, null);
    if (storedPayment) {
      await paymentServiceImpl.executeTransaction(storedPayment);
    }
  },
);

Then(
  "the user with email: {string} has {int} less on their account",
  async (emailLess: string, int: number) => {
    const firstUser = await User.findOne({ where: { email: emailLess } });
    if (!firstUser) {
      throw Error();
    }
    equal(firstUser.amount, initials[emailLess] - int);
  },
);

Then(
  "the user with email: {string} has {int} more on their account",
  async (emailMore: string, int: number) => {
    const secondUser = await User.findOne({ where: { email: emailMore } });
    if (!secondUser) {
      throw Error();
    }
    equal(secondUser.amount, initials[emailMore] + int);
  },
);
