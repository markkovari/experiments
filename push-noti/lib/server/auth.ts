import { cookies } from "next/headers";
import { randomBytes, scryptSync, timingSafeEqual } from "node:crypto";
import { one, run } from "@/lib/db";
import { json } from "./http";

export type User = { id: number; name: string };
const COOKIE = "sid";

const hash = (pw: string, salt = randomBytes(16).toString("hex")) => `${salt}:${scryptSync(pw, salt, 32).toString("hex")}`;
const verify = (pw: string, stored: string) => {
  const [salt, h] = stored.split(":");
  return timingSafeEqual(Buffer.from(h, "hex"), scryptSync(pw, salt, 32));
};

/** shortcut: the first login with a new name registers it; add a separate signup if that matters. */
export function loginOrRegister(name: string, pw: string): number | undefined {
  const row = one<{ id: number; pw: string }>("select id, pw from users where name = ?", name);
  if (!row) return Number(run("insert into users (name, pw) values (?, ?)", name, hash(pw)).lastInsertRowid);
  return verify(pw, row.pw) ? row.id : undefined;
}

export async function startSession(userId: number, secure: boolean) {
  const token = randomBytes(32).toString("hex");
  run("insert into sessions (token, user_id) values (?, ?)", token, userId);
  (await cookies()).set(COOKIE, token, { httpOnly: true, sameSite: "lax", secure, path: "/", maxAge: 60 * 60 * 24 * 30 });
}

export async function endSession() {
  const jar = await cookies();
  const token = jar.get(COOKIE)?.value;
  if (token) run("delete from sessions where token = ?", token);
  jar.delete(COOKIE);
}

export async function currentUser(): Promise<User | undefined> {
  const token = (await cookies()).get(COOKIE)?.value;
  return token ? one<User>("select u.id, u.name from sessions s join users u on u.id = s.user_id where s.token = ?", token) : undefined;
}

/** Runs `handler` with the logged-in user, or answers 401. */
export async function withUser(handler: (user: User) => Promise<Response>): Promise<Response> {
  const user = await currentUser();
  return user ? handler(user) : json({ error: "unauthorized" }, 401);
}
