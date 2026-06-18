import { createClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";
import { JobRunnerService } from "./gen/jobrunner/v1/jobrunner_pb.js";

// baseUrl "/" → requests go to the same origin; the Vite dev server proxies
// the Connect paths to the Rust backend (see vite.config.ts). For a
// standalone production build, point this at the server's origin instead.
const transport = createConnectTransport({
  baseUrl: import.meta.env.VITE_API_BASE_URL ?? "/",
});

export const client = createClient(JobRunnerService, transport);
