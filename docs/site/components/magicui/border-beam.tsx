"use client";

import React from "react";

import { cn } from "@/lib/utils";

interface BorderBeamProps {
  className?: string;
  size?: number;
  duration?: number;
  borderWidth?: number;
  delay?: number;
}

/** Magic UI: BorderBeam — orbiting gradient beam along a container border. */
export function BorderBeam({
  className,
  size = 50,
  duration = 6,
  borderWidth = 1.5,
  delay = 0,
}: BorderBeamProps) {
  return (
    <div
      className="pointer-events-none absolute inset-0 rounded-[inherit] border border-transparent [mask-clip:padding-box,border-box] [mask-composite:intersect] [mask-image:linear-gradient(transparent,transparent),linear-gradient(#000,#000)]"
      style={{ borderWidth }}
    >
      <div
        className={cn(
          "absolute aspect-square rounded-full bg-gradient-to-l from-transparent via-vigil-sky to-transparent",
          className,
        )}
        style={{
          width: size,
          offsetPath: `rect(0 auto auto 0 round ${size}px)`,
          animation: `border-beam-spin ${duration}s linear infinite`,
          animationDelay: `${delay}s`,
        }}
      />
      <style>{`
        @keyframes border-beam-spin {
          to { offset-distance: 100%; }
        }
      `}</style>
    </div>
  );
}
