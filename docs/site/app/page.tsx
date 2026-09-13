import Link from "next/link";

import { AuroraText } from "@/components/magicui/aurora-text";
import { BorderBeam } from "@/components/magicui/border-beam";
import { ShimmerButton } from "@/components/magicui/shimmer-button";
import { NumberTicker } from "@/components/magicui/number-ticker";
import { pages } from "@/lib/pages";

export default function Home() {
  return (
    <main className="mx-auto max-w-5xl px-6 py-16">
      <p className="text-sm uppercase tracking-[0.3em] text-vigil-sky">
        VigilFYR documentation
      </p>
      <h1 className="mt-4 text-4xl font-bold leading-tight sm:text-5xl">
        The <AuroraText>AI-agent guard</AuroraText>.
        <br />
        Every read. Every write. Every command.
      </h1>
      <p className="mt-6 max-w-2xl text-zinc-400">
        Vigil blocks AI coding agents from sensitive areas of your machine -
        secrets, <code>.env</code>, keys, <code>.ssh</code>, <code>.aws</code>,
        wallets, system dirs. Native hooks for supported agents, filesystem
        audit and the <code>vigil shim</code> wrapper for everything else.
        Proxy-proof: it guards local tool actions, so which LLM API or router
        you use is irrelevant.
      </p>

      <div className="mt-8 flex flex-wrap items-center gap-4">
        <Link href="/getting-started">
          <ShimmerButton
            background="linear-gradient(110deg, #0ea5e9, 45%, #8b5cf6, 55%, #ec4899)"
            className="px-6 py-3 text-base"
          >
            Get started
          </ShimmerButton>
        </Link>
        <a
          className="text-sm text-zinc-400 underline decoration-dotted hover:text-zinc-200"
          href="https://github.com/PotenFYR-Studios/VigilFYR"
        >
          GitHub →
        </a>
      </div>

      <div className="mt-10 grid grid-cols-3 gap-4 text-center">
        <div className="rounded-xl border border-zinc-800 p-4">
          <NumberTicker value={2} className="text-2xl font-bold text-vigil-sky" />
          <p className="mt-1 text-xs text-zinc-500">exit code: deny</p>
        </div>
        <div className="rounded-xl border border-zinc-800 p-4">
          <NumberTicker value={6} className="text-2xl font-bold text-vigil-violet" />
          <p className="mt-1 text-xs text-zinc-500">agents with hooks</p>
        </div>
        <div className="rounded-xl border border-zinc-800 p-4">
          <NumberTicker value={15} className="text-2xl font-bold text-vigil-pink" />
          <p className="mt-1 text-xs text-zinc-500">built-in core rules</p>
        </div>
      </div>

      <div className="mt-12 grid gap-4 sm:grid-cols-2">
        {pages.map((p) => (
          <Link
            key={p.href}
            href={p.href}
            className="group relative overflow-hidden rounded-xl border border-zinc-800 p-5 transition hover:border-zinc-600"
          >
            <BorderBeam size={80} duration={9} />
            <h2 className="text-lg font-semibold group-hover:text-vigil-sky">
              {p.title}
            </h2>
            <p className="mt-2 text-sm text-zinc-400">{p.blurb}</p>
          </Link>
        ))}
      </div>
    </main>
  );
}
