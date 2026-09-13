import type { ClassValue } from "./types";

/** Class name combiner (clsx/tailwind-merge compatible signature). */
export function cn(...inputs: ClassValue[]): string {
  return inputs.filter(Boolean).join(" ");
}
