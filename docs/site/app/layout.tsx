import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Vigil Docs — the AI-agent guard",
  description:
    "Block AI coding agents from reading, writing, searching, or executing in sensitive areas. Hooks for Claude Code, Codex, Gemini CLI, Cursor, OpenCode, Hermes; audit + shim for everything else.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-vigil-surface text-zinc-100 antialiased">
        {children}
      </body>
    </html>
  );
}
