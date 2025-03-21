import supertest from "supertest";
import { describe, expect, it } from "vitest";
import { app } from "../app";
import { User } from "../src/infrastructure/database/models/User";

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

  it("findById when user is present send back 200 and the user", async () => {
    const createUser = { firstName: "First", lastName: "User" };
    const createResponse = await api
      .post("/api/v1/users")
      .set("Content-type", "application/json")
      .send(JSON.stringify(createUser));
    const createdId = createResponse.body.id;
    //TODO: should be created link

    const response = await api.get(`/api/v1/users/${createdId}`);
    expect(response.statusCode).toBe(200);
    expect(response.body.firstName).toEqual("First");
    expect(response.body.lastName).toEqual("User");
    expect(response.body.id).toEqual(createdId);
  });

  it("deleting a user when user is present send back 200 and deletes the user", async () => {
    const createdUser = await User.create({
      firstName: "First",
      lastName: "User",
    });

    const response = await api.delete(`/api/v1/users/${createdUser.id}`);
    expect(response.statusCode).toBe(200);

    const allUsersResponse = await api.get("/api/v1/users/");
    expect(allUsersResponse.statusCode).toBe(200);
    expect(allUsersResponse.body).toHaveLength(0);
  });

  it("updateing a user when user is present send back 200 and updates the users fields", async () => {
    const createdUser = await User.create({
      firstName: "First",
      lastName: "User",
    });

    const payload = { firstName: "FirstSon" };
    const response = await api
      .post(`/api/v1/users/${createdUser.id}`)
      .send(JSON.stringify(payload))
      .set("Content-type", "application/json");
    expect(response.statusCode).toBe(200);

    const updateUserResponse = await api.get(`/api/v1/users/${createdUser.id}`);
    expect(updateUserResponse.statusCode).toBe(200);
    expect(updateUserResponse.body.firstName).toBe("FirstSon");
    expect(updateUserResponse.body.lastName).toBe("User");
  });
});
