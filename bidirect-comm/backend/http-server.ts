import express from "express";
import cors from "cors";

const app = express();
app.use(cors());
app.use(express.json());

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

app.post("/fibonacci", (req, res) => {
  const { n } = req.body as { n: number };
  if (typeof n !== "number" || n < 0 || !Number.isInteger(n)) {
    res.status(400).json({ error: "n must be a non-negative integer" });
    return;
  }
  const result = fibonacci(n);
  res.json({ n, result });
});

app.post("/factorial", (req, res) => {
  const { n } = req.body as { n: number };
  if (typeof n !== "number" || n < 0 || !Number.isInteger(n)) {
    res.status(400).json({ error: "n must be a non-negative integer" });
    return;
  }
  const result = factorial(n).toString();
  res.json({ n, result });
});

app.listen(3001, () => console.log("HTTP server running on :3001"));
