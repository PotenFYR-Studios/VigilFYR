import { Shield, Terminal, Zap } from "lucide-react";
import { Link } from "react-router-dom";

import { MagicCard, NumberTicker } from "../components/magicui";
import { DOCS } from "../lib/pages";

export function Home() {
  return (
    <div className="dot-backdrop">
      <div className="relative mx-auto max-w-5xl px-6 pb-20 pt-16">
        <p className="eyebrow">
          <span className="eyebrow-accent">VigilFYR</span> docs / v0.1.0
        </p>
        <h1 className="grad-text mt-3 text-[clamp(2.3em,5.4vw,3.4em)] font-extrabold leading-[1.08] tracking-tight">
          The AI-agent guard
        </h1>
        <p className="mt-5 max-w-2xl text-[1.05em] text-muted">
          Local, synchronous enforcement for every read, write, search and command.
          Native hooks, transparent rules, masking and audit without a cloud gate.
        </p>
        <div className="mt-8 flex flex-wrap gap-3">
          <Link className="primary-button" to="/docs/getting-started">
            Get started
          </Link>
          <a className="secondary-button" href="https://github.com/PotenFYR-Studios/VigilFYR" target="_blank" rel="noopener noreferrer">
            View source
          </a>
        </div>
        <div className="mt-12 grid gap-4 sm:grid-cols-3">
          <MagicCard className="p-5">
            <Shield className="h-5 w-5 text-brand-sky" />
            <NumberTicker value={2} className="mt-3 block text-3xl font-bold text-brand-sky" />
            <p className="mt-1 text-xs text-faint">deny exit code</p>
          </MagicCard>
          <MagicCard className="p-5">
            <Terminal className="h-5 w-5 text-brand-violet" />
            <NumberTicker value={6} className="mt-3 block text-3xl font-bold text-brand-violet" />
            <p className="mt-1 text-xs text-faint">hooked agents</p>
          </MagicCard>
          <MagicCard className="p-5">
            <Zap className="h-5 w-5 text-brand-pink" />
            <NumberTicker value={26} className="mt-3 block text-3xl font-bold text-brand-pink" />
            <p className="mt-1 text-xs text-faint">built-in rules</p>
          </MagicCard>
        </div>
        <div className="mt-12 grid gap-[14px] sm:grid-cols-2 lg:grid-cols-3">
          {DOCS.slice(0, 6).map((page) => (
            <Link key={page.slug} to={`/docs/${page.slug}`} className="doc-card">
              <span className="card-title">{page.title}</span>
              <span className="card-desc">{page.description}</span>
            </Link>
          ))}
        </div>
      </div>
    </div>
  );
}
