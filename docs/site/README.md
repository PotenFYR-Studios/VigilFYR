# vigil-docs

Documentation site for VigilFYR - Next.js (static export) + Tailwind CSS +
Magic UI component style.

Pages mirror the markdown docs in `docs/`:

| Route | Source content |
| :--- | :--- |
| `/` | docs/index.md |
| `/getting-started` | docs/getting-started.md |
| `/rules` | docs/rules.md |
| `/masking` | docs/masking.md |
| `/agents` | docs/agents.md |
| `/extensions` | docs/extensions.md |
| `/configuration` | docs/configuration.md |
| `/daemon` | docs/daemon.md |
| `/security-model` | docs/security-model.md |

## Develop

```sh
cd docs/site
npm install
npm run dev
```

## Build (static export)

```sh
npm run build   # outputs to out/
```

Static export (`output: "export"`) is GitHub Pages-ready; set
`DOCS_BASE_PATH` when building under a repo subpath:

```sh
DOCS_BASE_PATH=/VigilFYR npm run build
```

Magic UI components live in `components/magicui/` (AuroraText, BorderBeam,
ShimmerButton, NumberTicker), matching the magicui.dev import style.
