import type { ApiError, MeResponse, SendResponse } from "@/lib/shared/api";
import type { UnreadState } from "@/lib/shared/push";

async function call<T>(path: string, body?: unknown): Promise<T | ApiError> {
  const res = await fetch(`/api/${path}`, body === undefined ? undefined : { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
  return res.json();
}

export const isError = (r: object): r is ApiError => "error" in r;

export const api = {
  me: () => call<MeResponse>("me") as Promise<MeResponse>,
  login: (name: string, password: string) => call<{ ok: true }>("login", { name, password }),
  logout: () => call<{ ok: true }>("logout", {}),
  subscribe: (sub: PushSubscriptionJSON) => call<{ ok: true }>("subscribe", sub),
  send: (to: string, body: string) => call<SendResponse>("send", { to, body }),
  read: (ids: number[] | undefined, endpoint: string | undefined) => call<UnreadState>("read", { ids, endpoint }),
};
