"use client";
import { Check, CheckCheck } from "lucide-react";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import type { Message } from "@/lib/shared/api";
import { cn } from "@/lib/utils";
import { Logo } from "./Logo";

const rtf = new Intl.RelativeTimeFormat(undefined, { numeric: "auto", style: "narrow" });
function ago(unixSeconds: number) {
  const s = Math.round(unixSeconds - Date.now() / 1000);
  for (const [unit, size] of [["day", 86400], ["hour", 3600], ["minute", 60]] as const) if (Math.abs(s) >= size) return rtf.format(Math.round(s / size), unit);
  return "now";
}

export function Inbox({ messages, unread, onRead }: { messages: Message[]; unread: number; onRead: (ids?: number[]) => void }) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <h2 className="flex items-center gap-2 text-lg font-semibold">
          Inbox
          <Badge className={cn("rounded-full", unread ? "bg-fuchsia-500 text-white shadow-[0_0_14px] shadow-fuchsia-500/60" : "bg-white/10 text-muted-foreground")}>
            {unread} unread
          </Badge>
        </h2>
        {unread > 0 && (
          <Button variant="ghost" size="sm" onClick={() => onRead()}>
            <CheckCheck /> Mark all read
          </Button>
        )}
      </CardHeader>
      <CardContent>
        {messages.length === 0 ? (
          <div className="flex flex-col items-center gap-3 py-8 text-center text-muted-foreground">
            <Logo className="size-12 opacity-60" />
            <p className="text-sm">No messages yet. Send one to yourself ✨</p>
          </div>
        ) : (
          <ul className="-mx-2 flex flex-col gap-1">
            {messages.map((m) => (
              <li key={m.id} className={cn("flex items-start gap-3 rounded-xl p-2 transition-colors", !m.read && "bg-white/[0.05]")}>
                <Avatar className="size-9 ring-1 ring-white/15">
                  <AvatarFallback className="bg-gradient-to-br from-fuchsia-500 to-indigo-500 text-xs font-semibold text-white">
                    {m.sender.slice(0, 2).toUpperCase()}
                  </AvatarFallback>
                </Avatar>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 text-sm">
                    <span className="truncate font-medium">{m.sender}</span>
                    <span className="shrink-0 text-xs text-muted-foreground">{ago(m.at)}</span>
                    {!m.read && <span className="size-2 shrink-0 rounded-full bg-fuchsia-400 shadow-[0_0_8px] shadow-fuchsia-400" aria-label="unread" />}
                  </div>
                  <p className={cn("text-sm break-words", m.read ? "text-muted-foreground" : "text-foreground")}>{m.body}</p>
                </div>
                {!m.read && (
                  <Button variant="ghost" size="icon-sm" onClick={() => onRead([m.id])} aria-label={`Mark #${m.id} read`}>
                    <Check />
                  </Button>
                )}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
