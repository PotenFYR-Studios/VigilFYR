import { Menu, X } from "lucide-react";
import { Link, useLocation } from "react-router-dom";
import { useState } from "react";

export function SiteHeader() {
  const location = useLocation();
  const [open, setOpen] = useState(false);
  const docsActive = location.pathname.startsWith("/docs");

  return (
    <header className="site-header">
      <Link to="/" className="brand" aria-label="VigilFYR documentation home">
        <span>
          VigilFYR<span className="brand-dot">.</span>
          <span className="mono-label text-muted">docs</span>
        </span>
      </Link>
      <nav className="nav-desktop" aria-label="Primary">
        <Link to="/" className={`nav-link${location.pathname === "/" ? " active" : ""}`}>
          Home
        </Link>
        <Link to="/docs" className={`nav-link${docsActive ? " active" : ""}`}>
          Docs
        </Link>
        <a
          href="https://github.com/PotenFYR-Studios/VigilFYR"
          className="nav-link"
          target="_blank"
          rel="noopener noreferrer"
        >
          GitHub
        </a>
      </nav>
      <button
        type="button"
        className="mobile-toggle"
        aria-expanded={open}
        aria-label="Toggle navigation menu"
        onClick={() => setOpen((value) => !value)}
      >
        {open ? <X className="h-4 w-4" /> : <Menu className="h-4 w-4" />}
      </button>
      {open && (
        <nav className="nav-mobile" aria-label="Mobile">
          <Link to="/" className="nav-link" onClick={() => setOpen(false)}>
            Home
          </Link>
          <Link to="/docs" className="nav-link" onClick={() => setOpen(false)}>
            Docs
          </Link>
          <a
            href="https://github.com/PotenFYR-Studios/VigilFYR"
            className="nav-link"
            target="_blank"
            rel="noopener noreferrer"
          >
            GitHub
          </a>
        </nav>
      )}
    </header>
  );
}

export function SiteFooter() {
  return (
    <footer className="site-footer">
      <div className="sf-inner">
        <div>
          <Link to="/" className="brand">
            VigilFYR<span className="brand-dot">.</span>docs
          </Link>
          <p className="sf-tagline">
            Local-first enforcement for AI coding agents. Transparent rules, no cloud gate.
          </p>
        </div>
        <div className="sf-links">
          <Link to="/docs">Documentation</Link>
          <a href="https://github.com/PotenFYR-Studios/VigilFYR" target="_blank" rel="noopener noreferrer">
            GitHub
          </a>
          <a href="https://potenfyr.in/" target="_blank" rel="noopener noreferrer">
            potenfyr.in
          </a>
        </div>
      </div>
      <div className="sf-legal">
        <span>© 2026 PotenFYR Studios. Apache-2.0 with Commons Clause.</span>
        <span>Local policy. Visible decisions.</span>
      </div>
    </footer>
  );
}
