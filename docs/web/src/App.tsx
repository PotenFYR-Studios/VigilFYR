import { Route, Routes } from "react-router-dom";

import { SiteFooter, SiteHeader } from "./components/site-chrome";
import { Home } from "./pages/Home";
import { DocsHub } from "./pages/DocsHub";
import { DocRoute } from "./pages/DocRoute";
import { NotFound } from "./pages/NotFound";

export function App() {
  return (
    <div className="flex min-h-screen flex-col">
      <SiteHeader />
      <main className="flex-1">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/docs" element={<DocsHub />} />
          <Route path="/docs/:slug" element={<DocRoute />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </main>
      <SiteFooter />
    </div>
  );
}
