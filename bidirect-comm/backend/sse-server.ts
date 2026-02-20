import express, { Request, Response } from "express";
import cors from "cors";

const app = express();
app.use(cors());
app.use(express.json());

function* fibonacciSequence(upTo: number): Generator<number> {
  let a = 0, b = 1;
  for (let i = 0; i <= upTo; i++) {
    yield a;
    [a, b] = [b, a + b];
  }
}

function* factorialSequence(upTo: number): Generator<string> {
  let result = 1n;
  yield result.toString();
  for (let i = 1; i <= upTo; i++) {
    result *= BigInt(i);
    yield result.toString();
  }
}

function sseHeaders(res: Response) {
  res.setHeader("Content-Type", "text/event-stream");
  res.setHeader("Cache-Control", "no-cache");
  res.setHeader("Connection", "keep-alive");
  res.flushHeaders();
}

// Client sends n via query param; server streams back fib(0)..fib(n) one by one
app.get("/fibonacci/stream", (req: Request, res: Response) => {
  const n = parseInt(req.query.n as string, 10);
  if (isNaN(n) || n < 0) {
    res.status(400).json({ error: "n must be a non-negative integer" });
    return;
  }

  sseHeaders(res);
  let index = 0;
  for (const value of fibonacciSequence(n)) {
    res.write(`data: ${JSON.stringify({ index, value })}\n\n`);
    index++;
  }
  res.write("data: [DONE]\n\n");
  res.end();
});

// Streams factorial(0)..factorial(n) one by one
app.get("/factorial/stream", (req: Request, res: Response) => {
  const n = parseInt(req.query.n as string, 10);
  if (isNaN(n) || n < 0) {
    res.status(400).json({ error: "n must be a non-negative integer" });
    return;
  }

  sseHeaders(res);
  let index = 0;
  for (const value of factorialSequence(n)) {
    res.write(`data: ${JSON.stringify({ index, value })}\n\n`);
    index++;
  }
  res.write("data: [DONE]\n\n");
  res.end();
});

app.listen(3003, () => console.log("SSE server running on :3003"));
