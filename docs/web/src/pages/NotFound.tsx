import { Link } from "react-router-dom";

export function NotFound() {
  return (
    <div className="mx-auto max-w-3xl px-6 py-24 text-center">
      <p className="eyebrow">404</p>
      <h1 className="grad-text mt-3 text-4xl font-extrabold">Page not found</h1>
      <p className="mt-4 text-muted">The page moved or never existed.</p>
      <Link className="primary-button mt-8 inline-flex" to="/docs">
        Go to documentation
      </Link>
    </div>
  );
}
