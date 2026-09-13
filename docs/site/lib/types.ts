export type ClassValue =
  | string
  | null
  | false
  | undefined
  | ClassValue[]
  | Record<string, boolean | null | undefined>;
