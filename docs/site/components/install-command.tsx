"use client";

import { useState } from "react";

const CMD =
  "curl -fsSL https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/master/install.sh | sh";

export function InstallCommand() {
  const [copied, setCopied] = useState(false);

  return (
    <div className="my-4 flex items-center gap-3 overflow-x-auto rounded-lg border border-zinc-800 bg-black/60 p-4">
      <code className="whitespace-nowrap text-sm text-sky-300">{CMD}</code>
      <button
        className="ml-auto shrink-0 rounded-md border border-zinc-700 px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200"
        onClick={async () => {
          await navigator.clipboard.writeText(CMD);
          setCopied(true);
          setTimeout(() => setCopied(false), 1500);
        }}
      >
        {copied ? "copied" : "copy"}
      </button>
    </div>
  );
}
