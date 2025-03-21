import { describe, expect, it } from "vitest";
import { Payment } from "../src/infrastructure/database/models/Payment";
import { User } from "../src/infrastructure/database/models/User";

describe("database", () => {
  describe("functionaly", () => {
    it("should be able to store users", async () => {
      const savedUser = await User.create({
        firstName: "Mark",
        lastName: "Kovari",
      });
      expect(savedUser).not.toBeNull();
    });

    it("should be able to store multiple users", async () => {
      const savedUser = await User.create({
        firstName: "Mark",
        lastName: "Kovari",
      });
      const savedUser2 = await User.create({
        firstName: "Mark",
        lastName: "Kovari",
      });
      const userAmount = await User.count();
      expect(userAmount).toBe(2);
    });

    it("should be able to create transfers from one account to another", async () => {
      const oldG = await User.create({
        firstName: "George",
        lastName: "Soros",
      });
      const me = await User.create({
        firstName: "Mark",
        lastName: "Kovari",
      });

      const payment = await Payment.create({
        amount: 10000000,
        fromId: oldG.id,
        toId: me.id,
      });
      expect(payment.fromId).toBe(oldG.id);
      expect(payment.toId).toBe(me.id);
    });
  });
});
