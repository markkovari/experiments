import { cn } from "@/lib/utils";

export function Logo({ className }: { className?: string }) {
  return (
    <span className={cn("relative inline-block", className)}>
      <span aria-hidden className="absolute inset-0 rounded-[28%] bg-fuchsia-500/50 blur-xl" />
      {/* eslint-disable-next-line @next/next/no-img-element -- tiny static svg */}
      <img src="/icon.svg" alt="" className="relative size-full rounded-[28%] ring-1 ring-white/20" />
    </span>
  );
}
