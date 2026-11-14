"use client";
import { BellRing, LogOut } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { api, isError } from "@/lib/client/api";
import { currentEndpoint, enablePush, reconcile } from "@/lib/client/push";
import type { MeResponse } from "@/lib/shared/api";
import { cn } from "@/lib/utils";
import { Inbox } from "./components/Inbox";
import { LoginForm } from "./components/LoginForm";
import { Logo } from "./components/Logo";
import { SendForm } from "./components/SendForm";

export default function Home() {
  const [me, setMe] = useState<MeResponse>();
  const [status, setStatus] = useState("");

  const load = useCallback(async () => {
    const m = await api.me();
    setMe(m);
    if (m.user) await reconcile(m); // catches up devices that get no sync push (Safari/iOS)
  }, []);

  useEffect(() => {
    load();
    navigator.serviceWorker?.register("/sw.js");
    navigator.serviceWorker?.addEventListener("message", load);
    return () => navigator.serviceWorker?.removeEventListener("message", load);
  }, [load]);

  useEffect(() => {
    if (!status) return;
    const t = setTimeout(() => setStatus(""), 4000);
    return () => clearTimeout(t);
  }, [status]);

  if (!me) return <div className="grid min-h-dvh place-items-center"><Logo className="animate-float size-16" /></div>;
  if (!me.user) return <LoginForm onLogin={load} />;

  async function read(ids?: number[]) {
    const res = await api.read(ids, await currentEndpoint());
    if (isError(res)) return setStatus(res.error);
    await reconcile(res);
    await load();
  }

  async function send(to: string, body: string) {
    const res = await api.send(to, body);
    setStatus(isError(res) ? res.error : `Sent ${res.sent}/${res.total}`);
  }

  return (
    <main className="mx-auto flex min-h-dvh w-full max-w-lg flex-col gap-5 px-4 pt-[max(1.5rem,env(safe-area-inset-top))] pb-[max(6rem,env(safe-area-inset-bottom))]">
      <header className="flex items-center gap-3">
        <Logo className="size-11" />
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-xl font-semibold">Hi {me.user}</h1>
          <p className="text-glam text-sm font-medium">push-noti</p>
        </div>
        <Avatar className="size-9 ring-2 ring-fuchsia-400/50">
          <AvatarFallback className="bg-gradient-to-br from-fuchsia-500 to-indigo-500 text-xs font-semibold text-white">{me.user.slice(0, 2).toUpperCase()}</AvatarFallback>
        </Avatar>
        <Button variant="ghost" size="icon" aria-label="Log out" onClick={async () => { await api.logout(); await load(); }}>
          <LogOut />
        </Button>
      </header>

      <Card>
        <CardContent className="flex flex-wrap items-center gap-4">
          <div className="grid size-10 shrink-0 place-items-center rounded-xl bg-fuchsia-500/15 text-fuchsia-300"><BellRing className="size-5" /></div>
          <p className="min-w-40 flex-1 text-sm text-muted-foreground">Get pings on this device, even when the app is closed.</p>
          <Button size="lg" className="btn-glam w-full border-0 sm:w-auto" onClick={async () => setStatus(await enablePush(me.vapidPublicKey))}>Enable notifications</Button>
        </CardContent>
      </Card>

      <Inbox messages={me.messages} unread={me.badge} onRead={read} />
      <SendForm users={me.users} onSend={send} />

      <p role="status"
        className={cn("fixed inset-x-0 bottom-[max(1.25rem,env(safe-area-inset-bottom))] mx-auto w-fit max-w-[90vw] rounded-full border border-white/15 bg-white/10 px-4 py-2 text-sm shadow-xl backdrop-blur-xl transition-all duration-300",
          status ? "translate-y-0 opacity-100" : "pointer-events-none translate-y-4 opacity-0")}>
        {status}
      </p>
    </main>
  );
}
