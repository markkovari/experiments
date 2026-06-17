import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// In dev, proxy Connect RPC calls to the Rust server so the browser makes
// same-origin requests (no CORS needed). Connect paths are POSTs under
// /<package>.<Service>/<Method>.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "/jobrunner.v1.JobRunnerService": {
        target: "http://127.0.0.1:8088",
        changeOrigin: true,
      },
    },
  },
});
