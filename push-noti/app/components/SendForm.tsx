"use client";
import { Send } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function SendForm({ users, onSend }: { users: string[]; onSend: (to: string, body: string) => void }) {
  return (
    <Card>
      <CardHeader>
        <h2 className="text-lg font-semibold">Send a push</h2>
      </CardHeader>
      <CardContent>
        <form action={(f) => onSend(String(f.get("to")), String(f.get("body")))} className="flex flex-col gap-4">
          <div className="grid gap-2">
            <Label htmlFor="to">To</Label>
            {/* Native select on purpose: phones show their own picker. Styled to match shadcn's Input. */}
            <select id="to" name="to"
              className="h-9 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 [&>option]:bg-popover">
              <option value="*">✨ everyone</option>
              {users.map((u) => <option key={u} value={u}>{u}</option>)}
            </select>
          </div>
          <div className="grid gap-2">
            <Label htmlFor="body">Message</Label>
            <Input id="body" name="body" defaultValue="hello" maxLength={200} />
          </div>
          <Button type="submit" size="lg" className="btn-glam border-0">
            <Send /> Send push
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
