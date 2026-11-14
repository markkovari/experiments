// Real-browser e2e for everything except the vendor push service (FCM/APNs/Mozilla).
// Playwright's Chromium has no FCM keys, so pushManager.subscribe is stubbed to point at a
// local fake push service; the fake decrypts what the server sent, and CDP delivers it to the real SW.
import { test, expect, type Browser, type Page } from "@playwright/test";
import { execSync } from "node:child_process";
import { createECDH, randomBytes } from "node:crypto";
import { createServer } from "node:https";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
// @ts-expect-error no types
import ece from "http_ece";

// Fresh user names per attempt, so a retry does not inherit the previous attempt's messages.
const u = (name: string) => `${name}-${test.info().retry}-${test.info().workerIndex}`;

type Received = { path: string; headers: Record<string, unknown>; body: Buffer };
const received: Received[] = [];
const dir = mkdtempSync(join(tmpdir(), "push-e2e-"));
execSync(`openssl req -x509 -newkey rsa:2048 -nodes -subj /CN=localhost -days 1 -keyout ${dir}/k.pem -out ${dir}/c.pem`, { stdio: "ignore" });
const pushService = createServer({ key: readFileSync(`${dir}/k.pem`), cert: readFileSync(`${dir}/c.pem`) }, (req, res) => {
  const chunks: Buffer[] = [];
  req.on("data", (c) => chunks.push(c)).on("end", () => {
    received.push({ path: req.url!, headers: req.headers, body: Buffer.concat(chunks) });
    res.writeHead(201).end();
  });
});
test.beforeAll(() => new Promise<void>((r) => pushService.listen(3101, r)));
test.afterAll(() => pushService.close());

async function login(page: Page, name: string) {
  await page.goto("/");
  await page.getByLabel("Name").fill(name);
  await page.getByLabel("Password").fill("secret");
  await page.getByRole("button", { name: "Log in / sign up" }).click();
  await expect(page.getByRole("heading", { name: `Hi ${name}` })).toBeVisible();
}

// A logged-in device whose push subscription points at the fake push service under /<path>.
async function device(browser: Browser, name: string, path: string, { viaButton = false } = {}) {
  const ecdh = createECDH("prime256v1");
  const sub = { endpoint: `https://localhost:3101/${path}`, keys: { p256dh: ecdh.generateKeys().toString("base64url"), auth: randomBytes(16).toString("base64url") } };
  const ctx = await browser.newContext();
  await ctx.grantPermissions(["notifications"], { origin: test.info().project.use.baseURL });
  await ctx.addInitScript((sub) => {
    PushManager.prototype.subscribe = async () => ({ ...sub, toJSON: () => sub }) as unknown as PushSubscription;
    PushManager.prototype.getSubscription = async () => ({ ...sub, toJSON: () => sub }) as unknown as PushSubscription;
  }, sub);
  const page = await ctx.newPage();
  await login(page, name);
  if (viaButton) {
    await page.getByRole("button", { name: "Enable notifications" }).click();
    await expect(page.getByRole("status")).toHaveText("Notifications enabled on this device.");
  } else {
    // Same steps as the button, minus the click: in headless Chromium a click-driven requestPermission() makes the
    // page's later notifications vanish (headed Chromium and real Android are fine). Button path: own test below.
    await page.evaluate(async () => {
      await Notification.requestPermission();
      const sub = await (await navigator.serviceWorker.ready).pushManager.subscribe({ userVisibleOnly: true });
      await fetch("/api/subscribe", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(sub.toJSON()) });
    });
  }

  const cdp = await ctx.newCDPSession(page);
  const regs = new Promise<{ registrationId: string }[]>((r) => cdp.on("ServiceWorker.workerRegistrationUpdated", (e) => e.registrations.length && r(e.registrations)));
  await cdp.send("ServiceWorker.enable");
  const [{ registrationId }] = await regs;
  const origin = new URL(test.info().project.use.baseURL!).origin;
  return {
    page,
    /** Decrypted payloads the server pushed to this device, in order. */
    inbox: () => received.filter((r) => r.path === `/${path}`)
      .map((r) => JSON.parse(ece.decrypt(r.body, { version: "aes128gcm", privateKey: ecdh, authSecret: sub.keys.auth }).toString())),
    /** Hand a payload to this device's real service worker, as the browser would. */
    deliver: (payload: object) => cdp.send("ServiceWorker.deliverPushMessage", { origin, registrationId, data: JSON.stringify(payload) }),
    shown: () => page.evaluate(async () => (await (await navigator.serviceWorker.ready).getNotifications()).map((n) => n.body).sort()),
  };
}

async function send(page: Page, to: string, body: string) {
  await page.getByLabel("To").selectOption(to);
  await page.getByLabel("Message").fill(body);
  const [res] = await Promise.all([page.waitForResponse((r) => r.url().endsWith("/api/send")), page.getByRole("button", { name: "Send push" }).click()]);
  return res.json();
}

test("alice pushes a notification to bob", async ({ browser }) => {
  const bob = await device(browser, u("bob"), u("bob"));
  const alice = await (await browser.newContext()).newPage();
  await login(alice, u("alice"));
  expect(await send(alice, u("bob"), "hi bob")).toEqual({ sent: 1, total: 1 });

  // What the server sent to the push service: VAPID-signed, aes128gcm-encrypted for bob's keys.
  const msg = received.find((r) => r.path === `/${u("bob")}`)!;
  expect(msg.headers["content-encoding"]).toBe("aes128gcm");
  expect(msg.headers.authorization).toMatch(/^vapid t=.+, k=.+/);
  const [payload] = bob.inbox();
  expect(payload).toMatchObject({ type: "message", title: `From ${u("alice")}`, body: "hi bob", badge: 1, unread: [payload.id] });

  await bob.deliver(payload);
  await expect.poll(bob.shown).toEqual(["hi bob"]);
});

test("reading a message on one device clears it on the others", async ({ browser }) => {
  const phone = await device(browser, u("dave"), u("dave-phone"));
  const laptop = await device(browser, u("dave"), u("dave-laptop"));
  const erin = await (await browser.newContext()).newPage();
  await login(erin, u("erin"));
  await send(erin, u("dave"), "one");
  await send(erin, u("dave"), "two");
  expect(phone.inbox().map((p) => p.badge)).toEqual([1, 2]);
  // One at a time, like real pushes: CDP delivering two at the same instant makes Chrome drop one from getNotifications().
  for (const d of [phone, laptop]) {
    await d.deliver(d.inbox()[0]);
    await expect.poll(d.shown).toEqual(["one"]);
    await d.deliver(d.inbox()[1]);
    await expect.poll(d.shown).toEqual(["one", "two"]);
  }

  // Read "one" on the phone: its notification closes there, and the laptop gets a sync push.
  await phone.page.reload();
  await expect(phone.page.getByRole("heading", { name: "Inbox 2 unread" })).toBeVisible();
  const [one] = phone.inbox();
  await phone.page.getByRole("button", { name: `Mark #${one.id} read` }).click();
  await expect(phone.page.getByRole("heading", { name: "Inbox 1 unread" })).toBeVisible();
  await expect.poll(phone.shown).toEqual(["two"]);

  const sync = laptop.inbox().at(-1);
  expect(sync).toMatchObject({ type: "sync", badge: 1 });
  expect(phone.inbox().filter((p) => p.type === "sync")).toEqual([]); // the reader is not pushed to
  await laptop.deliver(sync);
  await expect.poll(laptop.shown).toEqual(["two"]);
});

test("the Enable notifications button subscribes this device", async ({ browser }) => {
  const carlos = await device(browser, u("carlos"), u("carlos"), { viaButton: true });
  expect(await send(carlos.page, u("carlos"), "to myself")).toEqual({ sent: 1, total: 1 });
  expect(carlos.inbox()).toMatchObject([{ type: "message", body: "to myself", badge: 1 }]);
});

test("wrong password is rejected", async ({ page }) => {
  await login(page, u("carol"));
  await page.getByRole("button", { name: "Log out" }).click();
  await page.getByLabel("Name").fill(u("carol"));
  await page.getByLabel("Password").fill("nope!");
  await page.getByRole("button", { name: "Log in / sign up" }).click();
  await expect(page.getByRole("status")).toHaveText("wrong password");
});
