import { LoginBody } from "@/lib/shared/api";
import { loginOrRegister, startSession } from "@/lib/server/auth";
import { json, readBody } from "@/lib/server/http";

export async function POST(req: Request) {
  const { data, error } = await readBody(req, LoginBody);
  if (error) return error;
  const userId = loginOrRegister(data.name, data.password);
  if (userId === undefined) return json({ error: "wrong password" }, 401);
  await startSession(userId, req.url.startsWith("https"));
  return json({ ok: true });
}
