import supertest from "supertest";
import { describe, expect, it } from "vitest";
import { app } from "../app";

const api = supertest(app);

describe("App", () => {
  it("all users should be empty", async () => {
    const response = await api.get("/api/v1/users");
    expect(response.statusCode).toBe(200);
  });
});
