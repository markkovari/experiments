import { all } from "@/lib/db";
import type { MeResponse } from "@/lib/shared/api";
import { currentUser } from "@/lib/server/auth";
import { json } from "@/lib/server/http";
import { recentMessages, unreadState } from "@/lib/server/messages";
import { vapidPublicKey } from "@/lib/server/push";

export async function GET() {
  const user = await currentUser();
  if (!user) return json({ vapidPublicKey } satisfies MeResponse);
  const users = all<{ name: string }>("select name from users order by name").map((u) => u.name);
  return json({ vapidPublicKey, user: user.name, users, messages: recentMessages(user.id), ...unreadState(user.id) } satisfies MeResponse);
}
