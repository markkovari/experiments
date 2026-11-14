import * as webpush from "web-push";
import { all, one, run } from "@/lib/db";
import type { PushPayload } from "@/lib/shared/push";

export type Subscription = { endpoint: string; userId: number; keys: webpush.PushSubscription };

// VAPID keys are generated once and live in the db, so there is zero config.
// "or ignore" + re-read: concurrent first boots (e.g. build workers) all end up with the same winner.
function vapidKeys(): webpush.VapidKeys {
  run("insert or ignore into kv (k, v) values ('vapid', ?)", JSON.stringify(webpush.generateVAPIDKeys()));
  return JSON.parse(one<{ v: string }>("select v from kv where k = 'vapid'")!.v);
}
const vapid = vapidKeys();
webpush.setVapidDetails(process.env.VAPID_SUBJECT ?? "mailto:admin@example.com", vapid.publicKey, vapid.privateKey);
export const vapidPublicKey = vapid.publicKey;

export function saveSubscription(userId: number, sub: webpush.PushSubscription) {
  run("insert into subs (endpoint, user_id, json) values (?, ?, ?) on conflict(endpoint) do update set user_id = excluded.user_id, json = excluded.json",
    sub.endpoint, userId, JSON.stringify(sub));
}

const toSubscription = (r: { endpoint: string; user_id: number; json: string }): Subscription => ({ endpoint: r.endpoint, userId: r.user_id, keys: JSON.parse(r.json) });

/** `to` is a user name or "*". */
export const subscriptionsOf = (to: string) =>
  all<{ endpoint: string; user_id: number; json: string }>("select s.endpoint, s.user_id, s.json from subs s join users u on u.id = s.user_id where ? = '*' or u.name = ?", to, to).map(toSubscription);

export const otherSubscriptionsOf = (userId: number, exceptEndpoint = "") =>
  all<{ endpoint: string; user_id: number; json: string }>("select endpoint, user_id, json from subs where user_id = ? and endpoint != ?", userId, exceptEndpoint).map(toSubscription);

// Apple revokes subscriptions whose pushes show no notification, so they cannot take silent sync pushes.
export const acceptsSilentPush = (s: Subscription) => !new URL(s.endpoint).hostname.endsWith("push.apple.com");

export async function pushTo(subs: Subscription[], payload: (s: Subscription) => PushPayload) {
  const results = await Promise.allSettled(subs.map((s) => webpush.sendNotification(s.keys, JSON.stringify(payload(s)))));
  let sent = 0;
  results.forEach((r, i) => {
    if (r.status === "fulfilled") sent++;
    // 404/410 = the browser dropped the subscription; forget it.
    else if (r.reason instanceof webpush.WebPushError && [404, 410].includes(r.reason.statusCode)) run("delete from subs where endpoint = ?", subs[i].endpoint);
    else console.error("push failed", subs[i].endpoint, r.reason);
  });
  return { sent, total: subs.length };
}
