import { z } from "zod";
import type { UnreadState } from "./push";

export const LoginBody = z.object({
  name: z.string().trim().regex(/^[\w.-]{1,32}$/, "name: 1-32 of [a-z0-9_.-]"),
  password: z.string().min(4, "password: 4+ chars"),
});
export const SubscribeBody = z.object({
  endpoint: z.url(),
  expirationTime: z.number().nullable().optional(),
  keys: z.object({ p256dh: z.string().min(1), auth: z.string().min(1) }),
});
export const SendBody = z.object({ to: z.string().default("*"), body: z.string().max(200).default("") });
/** ids omitted = mark everything read; endpoint = the reading device, which is not pushed to. */
export const ReadBody = z.object({ ids: z.array(z.number().int()).optional(), endpoint: z.string().optional() });

export type Message = { id: number; sender: string; body: string; read: boolean; at: number /* unix seconds */ };
export type MeResponse = { vapidPublicKey: string } & (
  | { user?: undefined }
  | ({ user: string; users: string[]; messages: Message[] } & UnreadState)
);
export type SendResponse = { sent: number; total: number };
export type ApiError = { error: string };
