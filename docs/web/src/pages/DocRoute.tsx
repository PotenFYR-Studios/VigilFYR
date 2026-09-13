import { useParams } from "react-router-dom";
import { Link } from "react-router-dom";

import { Markdown } from "../components/md";
import { DOCS, DOC_GROUPS } from "../lib/pages";
import { NotFound } from "./NotFound";

export function DocRoute() {
  const { slug = "" } = useParams();
  const page = DOCS.find((item) => item.slug === slug);
  if (!page) return <NotFound />;

  return (
    <div className="mx-auto flex max-w-6xl gap-10 px-6 py-12">
      <aside className="hidden w-56 shrink-0 md:block">
        {DOC_GROUPS.map((group) => (
          <div key={group} className="mb-5">
            <h2 className="side-title">{group}</h2>
            <nav className="mt-2 space-y-1">
              {DOCS.filter((item) => item.group === group).map((item) => (
                <Link key={item.slug} to={`/docs/${item.slug}`} className={`sidebar-link${item.slug === slug ? " active" : ""}`}>
                  {item.title}
                </Link>
              ))}
            </nav>
          </div>
        ))}
      </aside>
      <article className="min-w-0 flex-1">
        <Markdown content={page.body} />
      </article>
    </div>
  );
}
