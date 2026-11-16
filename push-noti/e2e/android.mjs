// Real-device e2e over adb (USB or Wi-Fi): real Chrome, real FCM, real Android notification.
// Usage: ANDROID_SERIAL=<serial> node e2e/android.mjs   (app must be running on :3200)
import { chromium } from "playwright";
import { execSync } from "node:child_process";
import assert from "node:assert/strict";

const PORT = 3200, APP = `http://localhost:${PORT}`;
const adb = (cmd) => execSync(`adb ${cmd}`, { encoding: "utf8" });
adb(`reverse tcp:${PORT} tcp:${PORT}`); // phone's localhost -> this machine; localhost is a secure context
adb("shell pm grant com.android.chrome android.permission.POST_NOTIFICATIONS");
adb("shell am force-stop com.android.chrome"); // an app window left over from a previous run covers the new tab
adb(`shell am start -a android.intent.action.VIEW -d ${APP} com.android.chrome`);
adb("forward tcp:9222 localabstract:chrome_devtools_remote");

// Chrome's DevTools socket appears a few seconds after a cold start.
let browser;
for (let i = 0; !browser; i++) browser = await chromium.connectOverCDP("http://localhost:9222").catch(async (e) => {
  if (i === 15) throw e;
  await new Promise((r) => setTimeout(r, 1000));
});
await (await browser.newBrowserCDPSession()).send("Browser.grantPermissions", { origin: APP, permissions: ["notifications"] });
const ctx = browser.contexts()[0];
const page = await ctx.newPage(); // a fresh foreground tab: an installed app's page may be hidden and unclickable
await page.goto(APP);

if (await page.getByRole("button", { name: "Log out" }).isVisible().catch(() => false)) await page.getByRole("button", { name: "Log out" }).click();
await page.getByLabel("Name").fill("android");
await page.getByLabel("Password").fill("android-e2e");
await page.getByRole("button", { name: "Log in / sign up" }).click();
await page.getByRole("button", { name: "Enable notifications" }).click();
await page.getByText("Notifications enabled on this device.").waitFor({ timeout: 20_000 });

// Fully close the app: nothing of Chrome (or an installed WebAPK) is running when the push arrives.
await browser.close();
adb("shell input keyevent KEYCODE_HOME");
await new Promise((r) => setTimeout(r, 5000)); // Chrome flushes cookies on backgrounding; a force-stop before that loses the login
adb("shell am force-stop com.android.chrome");
assert.equal(execSync("adb shell pidof com.android.chrome || true", { encoding: "utf8" }).trim(), "", "chrome still running");

// Send from a second user, straight through the API.
const login = await fetch(`${APP}/api/login`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ name: "alice", password: "alice-e2e" }) });
const cookie = login.headers.get("set-cookie").split(";")[0];
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Sends a message to "android", waits for it in the notification shade, taps it; returns the message id. */
async function sendAndTap(label) {
  const msg = `e2e-${label}-${Date.now()}`;
  const sent = await (await fetch(`${APP}/api/send`, { method: "POST", headers: { "content-type": "application/json", cookie }, body: JSON.stringify({ to: "android", body: msg }) })).json();
  assert.ok(sent.sent >= 1, `server push failed: ${JSON.stringify(sent)}`);
  for (let i = 0; !adb("shell dumpsys notification --noredact").includes(msg); i++) {
    assert.ok(i < 30, `"${msg}" never reached the notification shade`);
    await sleep(1000);
  }
  console.log(`PASS[${label}]: "${msg}" shown in the notification shade (sent ${sent.sent}/${sent.total})`);
  const me = await (await fetch(`${APP}/api/me`, { headers: { cookie: androidCookie } })).json();
  const id = me.messages.find((m) => m.body === msg).id;
  adb("shell cmd statusbar collapse"); // a quick-settings panel (e.g. Wi-Fi) left open hides the notifications
  adb("shell cmd statusbar expand-notifications");
  await sleep(1000);
  execSync(`bash e2e/tap.sh '${msg}'`, { stdio: "inherit" });
  return { id, msg };
}

/** The app page the tap opened: its URL is the deep link, that message is highlighted and shown in the modal. */
async function deepLinkedPage(id, body) {
  for (let i = 0; ; i++) {
    const b = await chromium.connectOverCDP("http://localhost:9222").catch(() => undefined);
    const page = b?.contexts().flatMap((c) => c.pages()).find((p) => p.url().endsWith(`/#m${id}`));
    if (page && await page.locator(`#m${id}[aria-current=true]`).isVisible().catch(() => false)
      && (await page.getByRole("dialog").textContent().catch(() => ""))?.includes(body)) return { b, page };
    await b?.close();
    assert.ok(i < 20, `no page at /#m${id} with that message highlighted and in the modal`);
    await sleep(1000);
  }
}

const androidLogin = await fetch(`${APP}/api/login`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ name: "android", password: "android-e2e" }) });
const androidCookie = androidLogin.headers.get("set-cookie").split(";")[0];

// Cold: nothing running, the tap opens a new window (the installed WebAPK if there is one) at the deep link.
const cold = await sendAndTap("cold");
let { b, page: app } = await deepLinkedPage(cold.id, cold.msg);
console.log(`PASS[cold]: tap opened ${app.url()} with #m${cold.id} highlighted and in the modal`);
await app.evaluate(() => { window.__noReload = true; });
await b.close();

// Warm: app open in the foreground, the tap navigates that same window to the new hash without reloading it.
const warm = await sendAndTap("warm");
({ b, page: app } = await deepLinkedPage(warm.id, warm.msg));
assert.equal(await app.evaluate(() => window.__noReload), true, "warm tap reloaded or replaced the page");
console.log(`PASS[warm]: tap moved the open app to ${app.url()} without a reload, modal shows the new message`);
await b.close();
process.exit(0);
