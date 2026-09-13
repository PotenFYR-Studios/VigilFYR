# Contributing to VigilFYR

Thanks for your interest in improving VigilFYR. Security tooling must be
extra careful with changes, so please read this guide.

## Setup

- Rust stable (see `rust-toolchain.toml`), no external services needed
- Bun 1.1+ only for the docs site
- `git clone` + `cargo build`

## Everyday commands

    cargo build                      # compile
    cargo test                       # unit, integration and E2E tests
    cargo fmt --check                # formatting
    cargo clippy --all-targets -- -D warnings
    cargo run -- doctor              # local policy and runtime health
    bash -n install.sh               # installer syntax
    shellcheck install.sh            # installer lint

CI runs fmt, clippy and tests on Linux, macOS and Windows. Run all three
checks before you push.

## Docs site

    cd docs/web && bun install       # deps
    cd docs/web && bun run dev       # dev server with HMR
    cd docs/web && bun run build     # production build

Docs content is plain markdown in `docs/*.md`; the site imports it at
build time, so editing a file there is all it takes. Keep `docs/index.md`,
the site route list (`docs/web/src/lib/pages.ts`), and README links in
sync when adding pages.

## Ground rules

1. **Fail open, never wedge the agent.** Hook execution must stay
   synchronous and local; any new path must preserve the exit-code
   contract (`0` allow, `2` deny, `3` ask).
2. **No secrets in code, logs, or tests.** Masking patterns must never
   emit more than the documented 4-character prefix.
3. **Rule changes keep first-match-wins ordering** in
   `rules/core.toml`: most specific first, and new rules get a stable
   id users can override.
4. **No shell interpolation for command execution.** Prefer direct
   process spawning with argument arrays.
5. Update documentation in the same change when behavior or policy changes.
6. ASCII only in source and docs; no em/en dashes or section signs.
7. Run all checks before every commit.
8. Never add AI attribution to commits, pull requests, tags, releases or
   changelogs.

## Pull requests

- One logical change per PR; include tests for behavior changes
- Parsers (rule files, config, archive handling) need fixtures first,
  including malformed-input cases
- Explain the security implications of any change touching rule
  evaluation, hook handling, masking, or the daemon/IPC surface
- CI must be green: fmt, clippy, tests on all three OSes

## Reporting issues

Use [GitHub Issues](https://github.com/PotenFYR-Studios/VigilFYR/issues)
for bugs and feature requests. For anything you believe is
**exploitable**, do not open a public issue: use [private vulnerability
reporting](https://github.com/PotenFYR-Studios/VigilFYR/security/advisories/new)
and see [SECURITY.md](SECURITY.md).

## License

By contributing you agree your contributions are licensed under the
Apache-2.0 with Commons Clause license covering the repository.
