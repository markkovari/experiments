import { SubscribeBody } from "@/lib/shared/api";
import { withUser } from "@/lib/server/auth";
import { json, readBody } from "@/lib/server/http";
import { saveSubscription } from "@/lib/server/push";

export const POST = (req: Request) => withUser(async (user) => {
  const { data, error } = await readBody(req, SubscribeBody);
  if (error) return error;
  saveSubscription(user.id, data);
  return json({ ok: true });
});
