// Screencast: the tempo worktime logger on the REAL running app. Seeds a few
// people, projects, and categories with time entries across the month via the
// API, then drives the SPA: a manager sees the whole team's distribution
// (donut by project, category + per-person bars, per-day series), flips the
// range (week/month/year) and the scope (Everyone/Mine), logs a live entry, and
// runs a pomodoro timer. Every chart is real report data from the component.
//
// Prereq: from repo root  `just host-tempo &`   (serves on :3040)
import { chromium } from "playwright";

const BASE = process.env.TEMPO_URL || "http://127.0.0.1:3040";
const OUT = new URL("./videos/tempo/", import.meta.url).pathname;
const W = 1120, H = 860;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---- seed via the API -------------------------------------------------------
const H_JSON = { "content-type": "application/json" };
async function api(path, method = "GET", body, token) {
  const h = { ...H_JSON }; if (token) h.authorization = `Bearer ${token}`;
  const r = await fetch(`${BASE}/api${path}`, { method, headers: h, body: body ? JSON.stringify(body) : undefined });
  return r.json().catch(() => ({}));
}
async function signup(email, role) {
  await api("/register", "POST", { email, password: "pw12345678", role });
  const l = await api("/login", "POST", { email, password: "pw12345678" });
  return l.access_token;
}
const month = new Date().toISOString().slice(0, 7); // current month, matches the SPA's default range
const day = (n) => `${month}-${String(n).padStart(2, "0")}`;

const admin = await signup("admin@acme.io", "admin");
await signup("boss@acme.io", "manager");
const boss = (await api("/login", "POST", { email: "boss@acme.io", password: "pw12345678" })).access_token;
const ada = await signup("ada@acme.io", "member");
const bo = await signup("bo@acme.io", "member");

// admin creates projects + categories
const P = {};
for (const [key, name] of [["APOLLO", "Apollo"], ["ZEPHYR", "Zephyr"], ["ORION", "Orion"]])
  P[name] = (await api("/projects", "POST", { key, name }, admin)).id;
const C = {};
for (const name of ["engineering", "sales", "design", "ops"])
  C[name] = (await api("/categories", "POST", { name }, admin)).id;

// spread entries across people, projects, categories, days
async function log(tok, proj, cat, d, mins) {
  await api("/entries", "POST", { project: P[proj], category: C[cat], minutes: mins, day: day(d) }, tok);
}
const plan = [
  [ada, "Apollo", "engineering", [[3,120],[5,180],[9,150],[12,90],[16,200],[19,160],[23,120]]],
  [ada, "Zephyr", "design", [[6,60],[13,90],[20,75]]],
  [bo, "Apollo", "engineering", [[4,140],[10,160],[17,130],[24,110]]],
  [bo, "Orion", "sales", [[2,90],[8,120],[15,140],[22,100]]],
  [boss, "Zephyr", "ops", [[7,80],[14,100],[21,60]]],
  [boss, "Orion", "engineering", [[11,90],[18,110]]],
];
for (const [tok, proj, cat, days] of plan) for (const [d, m] of days) await log(tok, proj, cat, d, m);

// ---- drive the SPA ----------------------------------------------------------
const browser = await chromium.launch({ headless: true });
const ctx = await browser.newContext({ viewport: { width: W, height: H }, recordVideo: { dir: OUT, size: { width: W, height: H } }, deviceScaleFactor: 2 });
const page = await ctx.newPage();
await page.goto(BASE);

try {
  // log in as the manager
  await page.fill("#email", "boss@acme.io");
  await page.fill("#password", "pw12345678");
  await page.click("#loginBtn");
  await page.locator("#app").waitFor({ timeout: 10000 });
  await sleep(1000);

  // the whole team's month: donut by project + category & per-person bars
  await page.click('#scopeSeg button[data-s="all"]');
  await sleep(2600);

  // flip the range
  await page.click('#ranges button[data-r="week"]'); await sleep(1600);
  await page.click('#ranges button[data-r="year"]'); await sleep(1600);
  await page.click('#ranges button[data-r="month"]'); await sleep(1400);

  // switch to my own view
  await page.click('#scopeSeg button[data-s="me"]'); await sleep(2000);

  // log a live entry — the charts update
  await page.selectOption("#proj", { label: "Orion" });
  await page.selectOption("#cat", { label: "engineering" });
  await page.fill("#mins", "45");
  await page.click("#logBtn");
  await sleep(2000);

  // run a pomodoro: start, let it tick, stop (logs an entry)
  await page.click("#pomo");
  await sleep(3200);
  await page.click("#pomo");
  await sleep(2200);
} finally {
  await ctx.close();
  await browser.close();
}
console.log("done");
