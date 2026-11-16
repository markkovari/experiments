"use client";
import { useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import type { Message } from "@/lib/shared/api";

/** The message a notification deep-linked to (/#m<id>). Native <dialog>: focus trap, Esc and backdrop for free. */
export function MessageDialog({ message, onClose }: { message: Message; onClose: () => void }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (!ref.current?.open) ref.current?.showModal(); // a second tap while open only swaps the content
  }, [message.id]);

  return (
    <dialog ref={ref} onClose={onClose} aria-label={`Message from ${message.sender}`}
      className="m-auto w-[min(90vw,24rem)] bg-transparent p-0 text-foreground backdrop:bg-black/60 backdrop:backdrop-blur-sm">
      <Card>
        <CardHeader>
          <CardTitle>From {message.sender}</CardTitle>
          <CardDescription>Opened from a notification · #{message.id}</CardDescription>
        </CardHeader>
        <CardContent><p className="break-words">{message.body}</p></CardContent>
        <CardFooter>
          <Button className="w-full" autoFocus onClick={() => ref.current?.close()}>Close</Button>
        </CardFooter>
      </Card>
    </dialog>
  );
}
