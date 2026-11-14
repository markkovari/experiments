// Service worker, bundled by esbuild into public/sw.js (npm run sw).
import { applyUnreadState, notificationTag, type PushPayload, type UnreadState } from "@/lib/shared/push";

declare const self: ServiceWorkerGlobalScope;

async function reconcile(state: UnreadState) {
  await applyUnreadState(self.registration, self.navigator, state);
  for (const w of await self.clients.matchAll({ type: "window" })) w.postMessage("refresh"); // an open app re-renders its inbox
}

self.addEventListener("push", (e) => {
  const payload = e.data?.json() as PushPayload | undefined;
  if (!payload) return;
  if (payload.type === "sync") return e.waitUntil(reconcile(payload)); // only sent to non-Apple devices
  // iOS revokes the subscription if a push does not show a notification.
  e.waitUntil(
    self.registration
      .showNotification(payload.title, { body: payload.body, tag: notificationTag(payload.id), icon: "/icon.svg", data: { id: payload.id } })
      .then(() => reconcile(payload)),
  );
});

// Tapping a notification reads that message: the server marks it, other devices get a sync push, the badge drops.
self.addEventListener("notificationclick", (e) => {
  e.notification.close();
  e.waitUntil((async () => {
    const endpoint = (await self.registration.pushManager.getSubscription())?.endpoint;
    const res = await fetch("/api/read", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ ids: [e.notification.data.id], endpoint }) });
    if (res.ok) await reconcile(await res.json());
    const [win] = await self.clients.matchAll({ type: "window" });
    await (win ? win.focus() : self.clients.openWindow("/"));
  })());
});

// Take over right away on deploy; otherwise an open app keeps running the old worker indefinitely.
self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));
