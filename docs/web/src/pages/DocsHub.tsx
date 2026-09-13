import { ArrowUpRight, FileText } from "lucide-react";
import { Link } from "react-router-dom";

import { DOCS, DOC_GROUPS } from "../lib/pages";

export function DocsHub() {
  return (
    <div className="mx-auto max-w-5xl px-6 pb-20 pt-14">
      <p className="eyebrow">
        <span className="eyebrow-accent">VigilFYR</span> documentation
      </p>
      <h1 className="grad-text mt-3 text-[clamp(2.2em,5vw,3.2em)] font-extrabold leading-[1.08] tracking-tight">
        VigilFYR Documentation
      </h1>
      <p className="mt-4 max-w-2xl text-[1.04em] text-muted">
        Installation, policy, masking, agents, daemon and extension guides.
      </p>
      {DOC_GROUPS.map((group) => (
        <section key={group} className="mt-12">
          <h2 className="side-title">{group}</h2>
          <div className="mt-4 grid gap-[14px] sm:grid-cols-2 lg:grid-cols-3">
            {DOCS.filter((page) => page.group === group).map((page) => (
              <Link key={page.slug} to={`/docs/${page.slug}`} className="doc-card">
                <div className="flex items-center justify-between">
                  <span className="card-icon">
                    <FileText className="h-5 w-5 text-brand-sky" />
                  </span>
                  <ArrowUpRight className="card-arrow h-4 w-4" />
                </div>
                <span className="card-title mt-2">{page.title}</span>
                <span className="card-desc">{page.description}</span>
              </Link>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
