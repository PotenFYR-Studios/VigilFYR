"use client";

import React from "react";

import { cn } from "@/lib/utils";

interface AuroraTextProps {
  children: React.ReactNode;
  className?: string;
}

/** Magic UI: AuroraText — gradient-animated inline text. */
export function AuroraText({ children, className }: AuroraTextProps) {
  return (
    <span
      className={cn(
        "animate-gradient-x bg-gradient-to-r from-vigil-sky via-vigil-violet to-vigil-pink bg-[length:200%_auto] bg-clip-text text-transparent",
        className,
      )}
    >
      {children}
    </span>
  );
}
