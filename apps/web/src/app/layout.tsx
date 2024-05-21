import "@/src/styles/globals.css";
import type { Metadata } from "next";
import { Toaster } from "@repo/ui/components";
// import { fontSans, fontMono } from "@/lib/fonts";
import TailwindResposivenessIndicator from "@repo/ui/components/TailwindResposivenessIndicator";
import type { PropsWithChildren } from "react";
import Provider from "./_provider";
import { cn } from "@repo/ui/cn";
import { fontMono, fontSans } from "../lib/fonts";

export const metadata: Metadata = {
  title: "Repo",
  description: "",
  themeColor: [
    { media: "(prefers-color-scheme: light", color: "white" },
    { media: "(prefers-color-scheme: dark", color: "black" },
  ],
};
// You might be wonder where is session provider for next-auth, well we necessarily don't need it in app router
export default function RootLayout({ children }: PropsWithChildren) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={cn(
          "min-h-screen bg-background font-sans antialiased",
          fontSans.variable,
          fontMono.variable
        )}
      >
        <Provider>
          {children}
          <TailwindResposivenessIndicator />
          <Toaster richColors/>
        </Provider>
      </body>
    </html>
  );
}
