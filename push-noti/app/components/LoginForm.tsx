"use client";
import { LogIn } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { api, isError } from "@/lib/client/api";
import { Logo } from "./Logo";

export function LoginForm({ onLogin }: { onLogin: () => void }) {
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);

  async function login(form: FormData) {
    setBusy(true);
    const res = await api.login(String(form.get("name")), String(form.get("password")));
    setBusy(false);
    if (isError(res)) return setStatus(res.error);
    onLogin();
  }

  return (
    <main className="mx-auto flex min-h-dvh w-full max-w-sm flex-col justify-center gap-8 px-4 py-[max(2rem,env(safe-area-inset-top))]">
      <div className="flex flex-col items-center gap-4 text-center">
        <Logo className="animate-float size-24" />
        <h1 className="text-glam text-4xl font-bold tracking-tight">push-noti</h1>
        <p className="text-muted-foreground">Notifications from the deep.</p>
      </div>
      <Card>
        <CardContent>
          <form action={login} className="flex flex-col gap-4">
            <div className="grid gap-2">
              <Label htmlFor="name">Name</Label>
              <Input id="name" name="name" required autoComplete="username" autoCapitalize="none" placeholder="jelly" />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="password">Password</Label>
              <Input id="password" name="password" type="password" required autoComplete="current-password" placeholder="••••" />
            </div>
            <Button type="submit" size="lg" className="btn-glam mt-2 border-0" disabled={busy}>
              <LogIn /> Log in / sign up
            </Button>
            <p role="status" className="min-h-5 text-center text-sm text-rose-300">{status}</p>
          </form>
        </CardContent>
      </Card>
      <p className="text-center text-xs text-muted-foreground">New name? Logging in creates the account.</p>
    </main>
  );
}
