import { SendBody, type SendResponse } from "@/lib/shared/api";
import { withUser } from "@/lib/server/auth";
import { json, readBody } from "@/lib/server/http";
import { createMessages, unreadState } from "@/lib/server/messages";
import { pushTo, subscriptionsOf } from "@/lib/server/push";

export const POST = (req: Request) => withUser(async (user) => {
  const { data, error } = await readBody(req, SendBody);
  if (error) return error;
  const text = data.body || "ping";
  const messageIds = createMessages(data.to, user.name, text);
  const states = new Map([...messageIds.keys()].map((id) => [id, unreadState(id)]));
  const result = await pushTo(subscriptionsOf(data.to), (s) => ({
    type: "message", id: messageIds.get(s.userId)!, title: `From ${user.name}`, body: text, ...states.get(s.userId)!,
  }));
  return json(result satisfies SendResponse);
});
