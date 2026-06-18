// Boot the vet-clinic backend + static SPA on PORT (default 3000). Seeds the
// three roles + a demo user per role, then prints the credentials.

import { buildApp } from "./app.js";

const PORT = Number(process.env.PORT ?? 3000);
const { app, demoUsers } = buildApp({ logger: true, serveStatic: true });

app
  .listen({ port: PORT, host: "0.0.0.0" })
  .then(() => {
    console.log(`\n🐾 vet-clinic up on http://localhost:${PORT}\n`);
    console.log("Demo logins (all under tenant acme-vet):");
    for (const u of demoUsers) {
      console.log(`  ${u.role.padEnd(10)}  ${u.email}  /  ${u.password}`);
    }
    console.log("\nEvery capability is an unmodified comp component running in-process via jco.\n");
  })
  .catch((err) => {
    app.log.error(err);
    process.exit(1);
  });
