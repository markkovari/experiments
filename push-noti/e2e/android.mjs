// Real-device e2e over adb (USB or Wi-Fi): real Chrome, real FCM, real Android notification.
// Usage: ANDROID_SERIAL=<serial> node e2e/android.mjs   (app must be running on :3200)
import { chromium } from "playwright";
import { execSync } from "node:child_process";
import assert from "node:assert/strict";

const PORT = 3200, APP = `http://localhost:${PORT}`;
const adb = (cmd) => execSync(`adb ${cmd}`, { encoding: "utf8" });
adb(`reverse tcp:${PORT} tcp:${PORT}`); // phone's localhost -> this machine; localhost is a secure context
adb("shell pm grant com.android.chrome android.permission.POST_NOTIFICATIONS");
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
adb("shell am force-stop com.android.chrome");
assert.equal(execSync("adb shell pidof com.android.chrome || true", { encoding: "utf8" }).trim(), "", "chrome still running");

// Send from a second user, straight through the API.
const login = await fetch(`${APP}/api/login`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ name: "alice", password: "alice-e2e" }) });
const cookie = login.headers.get("set-cookie").split(";")[0];
const msg = `e2e-${Date.now()}`;
const sent = await (await fetch(`${APP}/api/send`, { method: "POST", headers: { "content-type": "application/json", cookie }, body: JSON.stringify({ to: "android", body: msg }) })).json();
assert.ok(sent.sent >= 1, `server push failed: ${JSON.stringify(sent)}`);

for (let i = 0; i < 30; i++) {
  if (adb("shell dumpsys notification --noredact").includes(msg)) {
    console.log(`PASS: "${msg}" shown in the Android notification shade with Chrome not running beforehand (sent ${sent.sent}/${sent.total})`);
    process.exit(0);
  }
  await new Promise((r) => setTimeout(r, 1000));
}
console.error(`FAIL: "${msg}" never reached the notification shade`);
process.exit(1);
