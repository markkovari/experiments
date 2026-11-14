import type { Metadata, Viewport } from "next";
import "./globals.css";
import { Geist } from "next/font/google";
import { cn } from "@/lib/utils";

const geist = Geist({subsets:['latin'],variable:'--font-sans'});

export const metadata: Metadata = {
  title: "push-noti",
  appleWebApp: { capable: true, title: "push-noti", statusBarStyle: "black-translucent" },
  icons: { apple: "/apple-icon.png" },
};
// viewportFit cover + safe-area padding lets the cosmic background run under the notch in the installed app.
export const viewport: Viewport = { themeColor: "#1a1033", width: "device-width", initialScale: 1, viewportFit: "cover" };

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={cn("dark font-sans antialiased", geist.variable)}>
      <body className="text-foreground">{children}</body>
    </html>
  );
}
