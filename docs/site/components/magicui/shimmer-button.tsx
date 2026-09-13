"use client";

import React, { useRef } from "react";

import { cn } from "@/lib/utils";

interface ShimmerButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  shimmerColor?: string;
  background?: string;
  shimmerSize?: string;
  borderRadius?: string;
}

/** Magic UI: ShimmerButton - animated shine sweep across a gradient button. */
export function ShimmerButton({
  children,
  className,
  shimmerColor = "#ffffff",
  background = "linear-gradient(110deg, #0ea5e9, 45%, #8b5cf6, 55%, #ec4899)",
  shimmerSize = "0.05em",
  borderRadius = "0.75rem",
  ...props
}: ShimmerButtonProps) {
  const ref = useRef<HTMLButtonElement>(null);

  return (
    <button
      ref={ref}
      className={cn(
        "relative inline-flex cursor-pointer items-center justify-center font-semibold text-white transition-transform active:scale-95",
        className,
      )}
      style={{
        background,
        borderRadius,
        backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'%3E%3C/svg%3E"), ${background}`,
        backgroundSize: "200% 100%, 100% 100%",
        backgroundPosition: "-200% 0, 0 0",
        animation: "shimmer-move 3.5s linear infinite",
      }}
      {...props}
    >
      <span
        aria-hidden
        className="pointer-events-none absolute inset-0 animate-shine"
        style={{
          background: `linear-gradient(110deg, transparent 25%, ${shimmerColor} 50%, transparent 75%)`,
          backgroundSize: "200% 100%",
          mixBlendMode: "overlay",
          opacity: 0.35,
        }}
      />
      <span className="relative z-10 flex items-center gap-2">{children}</span>
      <style>{`
        @keyframes shimmer-move {
          to { background-position: 200% 0, 0 0; }
        }
      `}</style>
    </button>
  );
}
