import Link from "next/link";

import { pages } from "@/lib/pages";
import { cn } from "@/lib/utils";

export function DocShell({
  title,
  active,
  children,
}: {
  title: string;
  active: string;
  children: React.ReactNode;
}) {
  return (
    <div className="mx-auto flex max-w-6xl gap-10 px-6 py-12">
      <aside className="hidden w-56 shrink-0 md:block">
        <Link href="/" className="text-sm font-semibold text-vigil-sky">
          Vigil Docs
        </Link>
        <nav className="mt-4 space-y-1">
          {pages.map((p) => (
            <Link
              key={p.href}
              href={p.href}
              className={cn(
                "block rounded-md px-3 py-1.5 text-sm",
                p.href === active
                  ? "bg-zinc-800 text-white"
                  : "text-zinc-400 hover:text-zinc-200",
              )}
            >
              {p.title}
            </Link>
          ))}
        </nav>
      </aside>
      <article
        className="prose-invert max-w-3xl [&_code]:rounded [&_code]:bg-zinc-800 [&_code]:px-1 [&_code]:py-0.5 [&_code]:text-[0.9em] [&_h2]:mt-10 [&_h2]:text-2xl [&_h2]:font-bold [&_li]:mt-1 [&_p]:mt-4 [&_pre]:mt-4 [&_pre]:overflow-x-auto [&_pre]:rounded-lg [&_pre]:bg-black/60 [&_pre]:p-4 [&_pre]:text-sm [&_strong]:text-zinc-100 [&_code]:text-sky-300"
        style={{ color: "#a1a1aa" }}
      >
        <h1 className="text-3xl font-bold text-white">{title}</h1>
        {children}
      </article>
    </div>
  );
}
