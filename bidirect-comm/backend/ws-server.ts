import { WebSocketServer, WebSocket } from "ws";

const wss = new WebSocketServer({ port: 3002 });

function fibonacci(n: number): number {
  if (n <= 1) return n;
  let a = 0, b = 1;
  for (let i = 2; i <= n; i++) {
    [a, b] = [b, a + b];
  }
  return b;
}

function factorial(n: number): bigint {
  let result = 1n;
  for (let i = 2; i <= n; i++) result *= BigInt(i);
  return result;
}

wss.on("connection", (ws: WebSocket) => {
  console.log("WebSocket client connected");

  ws.on("message", (data) => {
    let n: number;
    let type: string;
    try {
      const parsed = JSON.parse(data.toString()) as { n: number; type?: string };
      n = parsed.n;
      type = parsed.type ?? "fibonacci";
    } catch {
      ws.send(JSON.stringify({ error: "Invalid JSON" }));
      return;
    }

    if (typeof n !== "number" || n < 0 || !Number.isInteger(n)) {
      ws.send(JSON.stringify({ error: "n must be a non-negative integer" }));
      return;
    }

    if (type === "factorial") {
      const result = factorial(n).toString();
      ws.send(JSON.stringify({ type, n, result }));
    } else {
      const result = fibonacci(n);
      ws.send(JSON.stringify({ type, n, result }));
    }
  });

  ws.on("close", () => console.log("WebSocket client disconnected"));
});

console.log("WebSocket server running on :3002");
