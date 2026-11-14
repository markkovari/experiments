// Shared by the server, the page and the service worker. No DOM or worker types here.

export type UnreadState = { badge: number; unread: number[]; latest: number };
export type MessagePush = UnreadState & { type: "message"; id: number; title: string; body: string };
export type SyncPush = UnreadState & { type: "sync" };
export type PushPayload = MessagePush | SyncPush;

export const notificationTag = (id: number) => `m${id}`;
const idOfTag = (tag: string) => Number(tag.slice(1)) || 0; // untagged/legacy notifications count as id 0

/** Notifications whose message is read. Ones newer than `latest` are kept: pushes can arrive out of order. */
export function staleNotifications<N extends { tag: string }>(shown: N[], state: UnreadState): N[] {
  return shown.filter((n) => {
    const id = idOfTag(n.tag);
    return id <= state.latest && !state.unread.includes(id);
  });
}

type NotificationSource = { getNotifications(): Promise<{ tag: string; close(): void }[]> };
type BadgeTarget = { setAppBadge?(count: number): Promise<void>; clearAppBadge?(): Promise<void> };

/** Close notifications of read messages and set the app-icon badge (installed apps on iOS, macOS, desktop Chrome/Edge). */
export async function applyUnreadState(source: NotificationSource | undefined, badge: BadgeTarget, state: UnreadState) {
  for (const n of staleNotifications((await source?.getNotifications()) ?? [], state)) n.close();
  // Rejects outside an installed app on some platforms; the badge is cosmetic, so ignore that.
  await (state.badge ? badge.setAppBadge?.(state.badge) : badge.clearAppBadge?.())?.catch(() => {});
}
