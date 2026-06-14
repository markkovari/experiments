// Example Fastify app guarded by the auth:identity contract.
//
// Public route, auth passthroughs, and two guarded routes demonstrating RBAC:
//   GET  /public            - no auth
//   POST /auth/register     - create account
//   POST /auth/login        - get token
//   GET  /auth/me           - whoami (any valid token)
//   POST /auth/logout       - revoke session
//   GET  /orders            - requires permission { orders, read }
//   POST /orders            - requires permission { orders, write }
//
// Run the auth stack first (see README), then:
//   AUTH_BASE_URL=http://localhost:8001 npm run dev

import Fastify from "fastify";
import { authPlugin } from "./auth-plugin.js";

const AUTH_BASE_URL = process.env.AUTH_BASE_URL ?? "http://localhost:8001";
const PORT = Number(process.env.PORT ?? 3000);

const app = Fastify({ logger: true });

await app.register(authPlugin, { baseUrl: AUTH_BASE_URL });

app.get("/public", async () => ({ ok: true, msg: "no auth needed" }));

// whoami: any valid token, no permission check (uses introspect, not verify).
app.get("/auth/me", async (request, reply) => {
  const h = request.headers.authorization;
  const token = h?.startsWith("Bearer ") ? h.slice(7).trim() : undefined;
  if (!token) return reply.code(401).send({ error: "missing_bearer_token" });
  try {
    return await app.auth.me(token);
  } catch (err) {
    const status = err && typeof err === "object" && "status" in err ? (err.status as number) : 503;
    return reply.code(status).send({ error: "unauthorized" });
  }
});

app.get(
  "/orders",
  { preHandler: app.requireAuth("orders", "read") },
  async (request) => ({
    orders: [{ id: 1, item: "widget" }],
    viewer: request.principal?.subject,
  }),
);

app.post(
  "/orders",
  { preHandler: app.requireAuth("orders", "write") },
  async (request, reply) => {
    reply.code(201);
    return { created: true, by: request.principal?.subject };
  },
);

try {
  await app.listen({ port: PORT, host: "0.0.0.0" });
  app.log.info(`auth base url: ${AUTH_BASE_URL}`);
} catch (err) {
  app.log.error(err);
  process.exit(1);
}
