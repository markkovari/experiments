import { equal, notEqual } from "node:assert";
import { Given, Then, When } from "@cucumber/cucumber";

import { Payment } from "../../src/infrastructure/database/models/Payment";
import { User } from "../../src/infrastructure/database/models/User";
import { paymentServiceImpl } from "../../src/infrastructure/services/PaymentServiceImpl";

let firstUser: User;
let firtUserInitial: number;
let secondUser: User;
let secondUserInitial: number;
let payment: Payment;

Given(
  "a user with email: a@gmail.com and {int} as amount",
  async (int: number) => {
    firstUser = await User.create({
      email: "a@gmail.com",
      firstName: "First",
      lastName: "Lastname",
      amount: int,
    });
    firtUserInitial = firstUser.amount;
  },
);

Given(
  "a user with email: b@gmail.com and {int} as amount",
  async (int: number) => {
    secondUser = await User.create({
      email: "b@gmail.com",
      firstName: "First",
      lastName: "Lastname",
      amount: int,
    });
    secondUserInitial = secondUser.amount;
  },
);

When(
  "the user with email: a@gmail.com pays to the user with: b@gmail.com with {int}",
  async (int: number) => {
    payment = await Payment.create({
      amount: int,
      fromId: firstUser.id,
      toId: secondUser.id,
    });
  },
);

Then("the payment is successfully registered", async () => {
  const storedPayment = await Payment.findOne({
    where: {
      fromId: firstUser.id,
      toId: secondUser.id,
    },
  });
  notEqual(storedPayment, null);
  if (storedPayment) {
    await paymentServiceImpl.executeTransaction(storedPayment);
  }
});

Then(
  "the user with email: a@gmail.com has {int} less on their account",
  async (int: number) => {
    const newFirst = await User.findByPk(firstUser.id);
    if (!newFirst) {
      throw Error();
    }
    firstUser = newFirst;
    equal(firstUser.amount, firtUserInitial - int);
  },
);

Then(
  "the user with email: b@gmail.com has {int} more on their account",
  async (int: number) => {
    const newSecondUser = await User.findByPk(secondUser.id);
    if (!newSecondUser) {
      throw Error();
    }
    secondUser = newSecondUser;
    equal(secondUser.amount, secondUserInitial + int);
  },
);
