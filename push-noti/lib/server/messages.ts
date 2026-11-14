import { all, one, run } from "@/lib/db";
import type { Message } from "@/lib/shared/api";
import type { UnreadState } from "@/lib/shared/push";

/** What every device needs to bring its notifications and icon badge in line with the server. */
export function unreadState(userId: number): UnreadState {
  const unread = all<{ id: number }>("select id from messages where user_id = ? and read_at is null order by id desc limit 50", userId).map((m) => m.id);
  const { badge, latest } = one<{ badge: number; latest: number }>(
    "select count(*) filter (where read_at is null) badge, coalesce(max(id), 0) latest from messages where user_id = ?", userId)!;
  return { badge, unread, latest };
}

export const recentMessages = (userId: number) =>
  all<{ id: number; sender: string; body: string; read: number; at: number }>(
    "select id, sender, body, read_at is not null read, created_at at from messages where user_id = ? order by id desc limit 20", userId,
  ).map((m): Message => ({ ...m, read: m.read === 1 }));

/** `to` is a user name or "*". Returns the new message id per recipient user id. */
export function createMessages(to: string, sender: string, body: string): Map<number, number> {
  const recipients = all<{ id: number }>("select id from users where ? = '*' or name = ?", to, to);
  return new Map(recipients.map((r) => [r.id, Number(run("insert into messages (user_id, sender, body) values (?, ?, ?)", r.id, sender, body).lastInsertRowid)]));
}

/** ids undefined = all. */
export function markRead(userId: number, ids?: number[]) {
  const list = ids ? JSON.stringify(ids) : null;
  run("update messages set read_at = unixepoch() where user_id = ? and read_at is null and (? is null or id in (select value from json_each(?)))", userId, list, list);
}
