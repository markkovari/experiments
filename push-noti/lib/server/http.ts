import type { z } from "zod";

export const json = (body: unknown, status = 200) => Response.json(body, { status });

/** Validates a JSON body; on failure `error` is a ready 400 response. */
export async function readBody<S extends z.ZodType>(req: Request, schema: S): Promise<{ data: z.infer<S>; error?: never } | { data?: never; error: Response }> {
  const parsed = schema.safeParse(await req.json().catch(() => ({})));
  return parsed.success ? { data: parsed.data } : { error: json({ error: parsed.error.issues.map((i) => i.message).join("; ") }, 400) };
}
