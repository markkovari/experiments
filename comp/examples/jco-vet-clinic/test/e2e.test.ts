// End-to-end flow for the vet-clinic example, driven via Fastify's app.inject
// (no network). Exercises the whole component stack: composed auth-guard
// (accounts/login/RBAC/authorize + audit), search-index, validate,
// notify-dispatch — all in-process over the shared KV.

import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import type { FastifyInstance } from "fastify";
import { buildApp } from "../src/app.js";

let app: FastifyInstance;

async function login(email: string, password: string): Promise<string> {
  const res = await app.inject({ method: "POST", url: "/auth/login", payload: { email, password } });
  assert.equal(res.statusCode, 200, `login ${email}: ${res.body}`);
  const tp = res.json() as { accessToken?: string; access_token?: string };
  return (tp.accessToken ?? tp.access_token)!;
}
function auth(token: string) {
  return { authorization: `Bearer ${token}` };
}

describe("vet-clinic e2e (composed components, in-process)", () => {
  before(() => {
    // Static disabled in tests (no public dir needed for inject).
    app = buildApp({ serveStatic: false }).app;
  });

  it("seeds 3 demo roles and they can all log in", async () => {
    await login("owner@acme-vet.test", "ownerpass1");
    await login("doctor@acme-vet.test", "doctorpass1");
    await login("admin@acme-vet.test", "adminpass1");
  });

  it("a pet-owner registers, logs in, and is a pet-owner", async () => {
    const reg = await app.inject({
      method: "POST",
      url: "/auth/register",
      payload: { email: "alice@acme-vet.test", password: "alicepass1", role: "pet-owner" },
    });
    assert.equal(reg.statusCode, 201, reg.body);
    const token = await login("alice@acme-vet.test", "alicepass1");
    const me = await app.inject({ method: "GET", url: "/auth/me", headers: auth(token) });
    assert.deepEqual((me.json() as { roles: string[] }).roles, ["pet-owner"]);
  });

  it("owner adds a pet (validated + indexed), then search finds it", async () => {
    const token = await login("alice@acme-vet.test", "alicepass1");
    const bad = await app.inject({ method: "POST", url: "/pets", headers: auth(token), payload: { name: "" } });
    assert.equal(bad.statusCode, 400, "empty name should fail validation");

    const ok = await app.inject({
      method: "POST",
      url: "/pets",
      headers: auth(token),
      payload: { name: "Rex", species: "dog" },
    });
    assert.equal(ok.statusCode, 201, ok.body);

    const found = await app.inject({ method: "GET", url: "/pets?q=Rex", headers: auth(token) });
    const pets = (found.json() as { pets: { name: string }[] }).pets;
    assert.ok(pets.some((p) => p.name === "Rex"), "search should find Rex");
  });

  it("owner books an appointment (notify fires, no throw)", async () => {
    const token = await login("alice@acme-vet.test", "alicepass1");
    const pets = (await app.inject({ method: "GET", url: "/pets", headers: auth(token) }).then((r) => r.json())) as { pets: { id: string }[] };
    const petId = pets.pets[0].id;
    const res = await app.inject({
      method: "POST",
      url: "/appointments",
      headers: auth(token),
      payload: { pet: petId, datetime: "2026-07-01T10:00" },
    });
    assert.equal(res.statusCode, 201, res.body);
  });

  it("owner is FORBIDDEN from writing a visit note (notes:write)", async () => {
    const token = await login("alice@acme-vet.test", "alicepass1");
    const res = await app.inject({
      method: "POST",
      url: "/appointments/appt_x/notes",
      headers: auth(token),
      payload: { text: "should be blocked" },
    });
    assert.equal(res.statusCode, 403, "pet-owner lacks notes:write");
  });

  it("doctor can list appointments and write a note", async () => {
    const doc = await login("doctor@acme-vet.test", "doctorpass1");
    const list = await app.inject({ method: "GET", url: "/appointments", headers: auth(doc) });
    assert.equal(list.statusCode, 200);
    // doctor writes a note on any appointment id (route-level perm is what we assert)
    const owner = await login("alice@acme-vet.test", "alicepass1");
    const appts = (await app.inject({ method: "GET", url: "/appointments", headers: auth(owner) }).then((r) => r.json())) as { appointments: { id: string }[] };
    const apptId = appts.appointments[0].id;
    const note = await app.inject({
      method: "POST",
      url: `/appointments/${apptId}/notes`,
      headers: auth(doc),
      payload: { text: "Healthy. Next visit in 6 months." },
    });
    assert.equal(note.statusCode, 201, note.body);
  });

  it("admin reads the audit trail (auth-guard recorded events)", async () => {
    const admin = await login("admin@acme-vet.test", "adminpass1");
    const res = await app.inject({ method: "GET", url: "/admin/audit", headers: auth(admin) });
    assert.equal(res.statusCode, 200, res.body);
    const events = (res.json() as { events: unknown[] }).events;
    assert.ok(Array.isArray(events) && events.length > 0, "audit trail should have events from all the auth activity");
  });

  it("an owner cannot reach the admin audit route", async () => {
    const token = await login("alice@acme-vet.test", "alicepass1");
    const res = await app.inject({ method: "GET", url: "/admin/audit", headers: auth(token) });
    assert.equal(res.statusCode, 403, "owner lacks audit:read");
  });
});
