import { defineConfig } from "@playwright/test";

// E2E_BASE_URL=https://<machine>.<tailnet>.ts.net:8443 targets an already-served app; its server must trust
// the fake push service's self-signed cert (NODE_TLS_REJECT_UNAUTHORIZED=0), so never point it at a real-device server.
const external = process.env.E2E_BASE_URL;

export default defineConfig({
  testDir: "e2e",
  use: { baseURL: external ?? "http://localhost:3100" },
  // channel "chromium" = new headless mode; the default headless-shell denies notification permission.
  projects: [{ name: "chromium", use: { browserName: "chromium", channel: "chromium" } }],
  webServer: external ? undefined : {
    command: "rm -f e2e.sqlite e2e.sqlite-wal e2e.sqlite-shm && npm run build && npx next start -p 3100",
    url: "http://localhost:3100/api/me",
    timeout: 120_000,
    // The fake push service in the test uses a self-signed cert; this only affects the test server.
    env: { DB_FILE: "e2e.sqlite", NODE_TLS_REJECT_UNAUTHORIZED: "0" },
  },
});
