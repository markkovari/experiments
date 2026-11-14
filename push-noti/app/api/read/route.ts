import { ReadBody } from "@/lib/shared/api";
import { withUser } from "@/lib/server/auth";
import { json, readBody } from "@/lib/server/http";
import { markRead, unreadState } from "@/lib/server/messages";
import { acceptsSilentPush, otherSubscriptionsOf, pushTo } from "@/lib/server/push";

export const POST = (req: Request) => withUser(async (user) => {
  const { data, error } = await readBody(req, ReadBody);
  if (error) return error;
  markRead(user.id, data.ids);
  const state = unreadState(user.id);
  // shortcut: Safari/iOS devices get no sync push (see acceptsSilentPush); they catch up when the app opens
  // or the next message arrives. Upgrade to Declarative Web Push if they must clear instantly.
  await pushTo(otherSubscriptionsOf(user.id, data.endpoint).filter(acceptsSilentPush), () => ({ type: "sync", ...state }));
  return json(state);
});
