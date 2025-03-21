import supertest from "supertest";
import { describe, expect, it } from "vitest";
import { app } from "../app";

const api = supertest(app);

describe("App", () => {
  it("all users should be empty", async () => {
    const response = await api.get("/api/v1/users");
    expect(response.statusCode).toBe(200);
    expect(response.body).toEqual([]);
  });

  it("creating a user should return with 201", async () => {
    const createUser = { firstName: "First", lastName: "User" };
    const response = await api
      .post("/api/v1/users")
      .set("Content-type", "application/json")
      .send(JSON.stringify(createUser));
    expect(response.statusCode).toBe(201);
  });

  it("creating a user should return with 201", async () => {
    const createUser = { firstName: "First", lastName: "User" };
    const response = await api
      .post("/api/v1/users")
      .set("Content-type", "application/json")
      .send(JSON.stringify(createUser));
    expect(response.statusCode).toBe(201);
  });

  it("creating a user when no users should add one user", async () => {
    const createUser = { firstName: "First", lastName: "User" };
    const response = await api
      .post("/api/v1/users")
      .set("Content-type", "application/json")
      .send(JSON.stringify(createUser));
    expect(response.statusCode).toBe(201);

    const allUsers = await api.get("/api/v1/users");

    expect(allUsers.body.length).toBe(1);
  });

  it("findById when no user is present send back 404", async () => {
    const response = await api.get("/api/v1/users/420");
    expect(response.statusCode).toBe(404);
  });
});
