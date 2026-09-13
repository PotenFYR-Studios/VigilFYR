<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:0ea5e9,50:8b5cf6,100:ec4899&height=220&section=header&text=VigilFYR&fontSize=52&fontColor=ffffff&fontAlignY=34&desc=The%20AI-agent%20guard.%20Every%20read.%20Every%20write.%20Every%20command.&descSize=18&descAlignY=55&animation=twinkling" width="100%" alt="VigilFYR banner"/>

[![Typing SVG](https://readme-typing-svg.demolab.com?font=Fira+Code&weight=600&size=20&pause=1200&color=8B5CF6&center=true&vCenter=true&width=800&lines=Block+AI+agents+from+your+secrets+%26+keys;Hooks+for+Claude+Code%2C+Codex%2C+Gemini+CLI+%26+more;Audit+watch+%2B+shim+for+everything+else;Proxy-proof%3A+guards+local+tool+actions;Live+TUI+report+of+every+blocked+event)](https://github.com/PotenFYR-Studios/VigilFYR)

[![CI](https://img.shields.io/github/actions/workflow/status/PotenFYR-Studios/VigilFYR/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=CI&labelColor=1c1e26&color=2ea043)](https://github.com/PotenFYR-Studios/VigilFYR/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/PotenFYR-Studios/VigilFYR?style=for-the-badge&logo=github&logoColor=white&label=Release&labelColor=1c1e26&color=eac54f)](https://github.com/PotenFYR-Studios/VigilFYR/releases)
[![Docs](https://img.shields.io/badge/Docs-VigilFYR%20docs-8b5cf6?style=for-the-badge&logo=readme&logoColor=white&labelColor=1c1e26)](https://potenfyr-studios.github.io/VigilFYR/)
[![License](https://img.shields.io/badge/License-Apache--2.0%20%2B%20Commons%20Clause-2ea043?style=for-the-badge&logo=apache&logoColor=white&labelColor=1c1e26)](https://github.com/PotenFYR-Studios/VigilFYR/blob/master/LICENSE)
[![Platforms](https://img.shields.io/badge/Platforms-linux%20%7C%20macOS%20%7C%20Windows-0ea5e9?style=for-the-badge&logo=linux&logoColor=white&labelColor=1c1e26)](#platform-support)
[![View](https://komarev.com/ghpvc/?username=PotenFYR-Studios-VigilFYR&color=ec4899&style=for-the-badge&label=VIEW&labelColor=1c1e26)](https://github.com/PotenFYR-Studios/VigilFYR)

[Overview](#overview) · [Install](#install) · [Quick Start](#quick-start) · [Docs](https://potenfyr-studios.github.io/VigilFYR/) · [Rules](#rules) · [Extensions](#extensions) · [Releases](https://github.com/PotenFYR-Studios/VigilFYR/releases)

```bash
curl -fsSL https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/master/install.sh | sh
```

</div>

---

## Overview

Vigil is a local guard for AI coding agents: it sits between your agent and your filesystem and blocks reads, writes, searches, and commands that touch sensitive areas - `.env` files, secrets directories, SSH and AWS credentials, private keys, crypto wallets, system paths. Hooks are auto-installed for agents that support them; a filesystem-watch audit mode and the `vigil shim` wrapper cover everything else.

Because it guards the tool actions on your machine, it is **proxy-proof**: it works identically behind any LLM API proxy or router (9router, LiteLLM, corporate gateways) - the model never sees what the tool never touched.

Built by **PotenFYR Studios**.

---

## Core Guarantees

- Every tool call is evaluated against the ruleset **before it executes**, with a strict exit-code contract: `0` allow, `2` deny, `3` ask.
- Verdicts are local and synchronous. Hook execution never waits on the daemon; a dead or restarting daemon never delays or breaks your agent.
- On daemon or ruleset error the hook **fails open** - a broken guard must never break your workflow. Calibration mode (`audit`) demotes every `deny` to `warn`, so nothing is ever blocked while you tune rules.
- Rules layer deterministically: built-in core ruleset → `~/.vigil/rules/*.toml` → `./.vigil/rules/*.toml`, overriding by rule id, so your local rules always win.
- Masking is opt-in and preserves a 4-character prefix so you can still tell which secret was matched - and it never emits more than that.
- Vigil inspects local tool actions, not network traffic. Which LLM API, proxy, or router you use is irrelevant: same rules, same verdicts, everywhere.

---

## Features

### Enforcement, three layers deep
- **Hooks (block before it runs)** - `vigil setup` detects installed agents and wires their native hook systems automatically. Every read, write, search, exec, and net action is checked first.
- **Audit watcher (warn, never block)** - a filesystem watcher records sensitive-area access by any process; for agents without hooks.
- **`vigil shim` (wrap anything)** - `vigil shim <command>…` puts arbitrary commands under the same rule evaluation.
- Enforcement where possible, visibility everywhere - no agent gets a free pass; only enforcement depth differs.

### Rules
- Built-in core ruleset: `.env` files, secrets dirs, `.ssh/`, `.aws/`, `.gnupg/`, key and credentials files, wallet directories, system paths - plus exec guards for `rm -rf /`, `sudo` system writes, and cloud metadata endpoints.
- Rule schema with severities and priorities; first match wins, most specific first.
- Built-in coverage now includes cloud/CI credentials, database configs and dumps, browser credential stores, system identity writes, agent config tampering, reverse shells, base64 shell pipelines, package-script exfiltration, and history/wipe attempts.
- Remote rule sync from the community feed on boot and `vigil reload`; `vigil rules update` refreshes on demand.
- Override any built-in rule by id from `~/.vigil/rules/` or `./.vigil/rules/`.

### Masking (opt-in, off by default)
- AWS keys, GitHub PATs, OpenAI keys, JWTs, private keys, and env values redacted in tool output, with a 4-character prefix preserved.
- Reusable custom patterns via `rules/patterns.toml` and extensions.

### Daemon, tray and updates
- `vigil daemon` owns event persistence, a local Unix-socket bus, tray integration, and update checks. Autostart and tray are opt-out config keys.
- `vigil update` self-updates the binary (semver check plus commits-behind, surfaced in the tray and TUI).

### Live report
- `vigil tui` is the default command: every verdict, who triggered it, which agent, and why - rule id and severity included - with a scrollable event log and export.

### Extensions
- Drop a `manifest.toml` into `~/.vigil/extensions/<name>/` to add your own rules, masking patterns, and event hooks. No fork required.

---

## Security Model

| Layer | Control |
|---|---|
| Hook path | Local, synchronous evaluation; exit codes `0`/`2`/`3`; fail-open only on daemon/ruleset error |
| Rule engine | Layered rulesets, id-based override, first-match-wins, severity tagging |
| Rules supply chain | Community feed sync, checksum-free but id-addressable - local files always override remote |
| Sensitive data | Masking opt-in, 4-char prefix cap, patterns never log matched secret bodies |
| Coverage | Hooks (enforce) + audit watcher + shim - universal visibility, best-effort enforcement |
| Failure mode | Audit mode demotes deny→warn; daemon unavailability never blocks tool calls |

Full threat model and honest limits: [docs/security-model.md](docs/security-model.md).

---

## Repository Layout

```
src/
  main.rs               CLI entrypoint (clap)
  lib.rs                library root
  engine/               decision engine: rule evaluation, verdicts
  rules.rs              rule model, layering, precedence
  config.rs             config.toml schema, load/save
  agents/               per-agent hook adapters (claude_code, codex,
                        gemini, cursor, opencode, hermes, generic)
  daemon.rs             background daemon, event persistence, tray
  ipc.rs                Unix-socket bus
  event.rs              event model
  log.rs                event log store
  mask.rs               masking patterns and redaction
  sync.rs               remote rule feed sync
  update.rs             self-update
  tui.rs                live report TUI
  tray.rs               tray integration
  cmd/                  subcommands: setup, daemon, intercept, shim,
                        rules, config, extension, agents, update
rules/
  core.toml             built-in ruleset
  patterns.toml         reusable masking patterns
tests/                  agents, intercept, masking pipeline, rules, shim, e2e
docs/                   markdown docs (imported by the docs site)
docs/site/              Next.js docs site, exported for GitHub Pages
.github/workflows/      ci, release, docs-pages, snake
install.sh              POSIX installer (checksum-verified)
install.ps1             PowerShell installer
```

---

## Install

OS and arch are detected and the download is checksum-verified:

```sh
curl -fsSL https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/master/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/master/install.ps1 | iex
```

The binary lands in `~/.local/bin` (add it to `PATH` if the installer says so). Release tarballs cover **Linux, macOS and Windows** on **x86_64 and arm64**, each with a `.sha256` sidecar.

## Quick Start

```sh
# Interactive wizard: OS check, agent auto-detect, per-agent mode,
# extra protected paths, masking opt-in, autostart + tray
vigil setup

# Or zero prompts, all defaults
vigil setup --defaults

# Check what your rules actually are
vigil rules list
vigil rules path

# Live report
vigil tui
```

That's it. Agents you had installed are now guarded; `vigil tui` shows the live report.

---

## Configuration

Config lives at `~/.vigil/config.toml`. A missing file means defaults. Read and write keys without editing the file:

```sh
vigil config get masking.enabled        # false
vigil config set general.mode audit     # calibrate without blocking
vigil config set agents.codex enforce
vigil config set masking.enabled true
```

| Key | Default | What it does |
| :--- | :--- | :--- |
| `general.enabled` | `true` | Master switch |
| `general.mode` | `enforce` | `enforce` or `audit` (audit never blocks) |
| `daemon.autostart` | `true` | Start the daemon on login |
| `daemon.tray` | `true` | Show the tray icon (update checks live here too) |
| `masking.enabled` | `false` | Opt-in sensitive-data masking |
| `masking.patterns` | all built-in | fourteen provider/token/URI patterns plus JWT, private keys, and env values |
| `rules.remote_update` | `true` | Sync community rules on boot / `vigil reload` |
| `agents.<id>` | `claude-code=enforce` | Per-agent mode: `off`, `audit`, `enforce` |

Rules layering: built-in core ruleset → `~/.vigil/rules/*.toml` → `./.vigil/rules/*.toml` (later files override earlier ones **by rule id**; within a file, order is priority - first match wins). `vigil reload` applies everything without a restart.

Full schema and cookbook: [Rules Reference](docs/rules.md) · [Configuration Reference](docs/configuration.md).

---

## Supported Agents

| Agent | Enforcement | Audit | Shim |
| :--- | :---: | :---: | :---: |
| Claude Code | 🪝 native hooks | ✅ | ✅ |
| Codex | 🪝 native hooks | ✅ | ✅ |
| Gemini CLI | 🪝 native hooks | ✅ | ✅ |
| Cursor | 🪝 native hooks | ✅ | ✅ |
| OpenCode | 🪝 native hooks | ✅ | ✅ |
| Hermes | 🪝 native hooks | ✅ | ✅ |
| anything else | - | ✅ | ✅ |

`vigil setup` auto-detects which of these are installed and configures the best mode for each; `vigil agents list|install|remove` manages hooks explicitly. Full matrix: [docs/agents.md](docs/agents.md).

---

## CLI Reference

```
vigil                       launch the TUI (default command)
vigil tui [--once]          live report; --once prints one snapshot
vigil setup [--defaults]    interactive (or promptless) setup wizard
vigil daemon [--no-tray]    run the background daemon
vigil rules list|path       inspect the active ruleset
vigil rules test <action> [path] [--command <cmd>] [--agent <id>]
                            dry-run the effective policy without logging an event
vigil doctor                read-only setup, agent, rule and masking health check
vigil rules update          refresh the remote community feed
vigil reload                reload rules + config without restart
vigil config get|set        read/write config keys
vigil agents list|install|remove   manage per-agent hooks
vigil shim <command>…       run a command under Vigil's wrapper
vigil extension             manage extensions
vigil update [--check]      self-update the binary
```

(`vigil intercept` is the internal hook entrypoint your agents call; you never invoke it by hand.)

---

## Updating

Vigil tells you when a new release ships (update checks ride along with the tray; opt out with `daemon.tray = false` / `vigil daemon --no-tray`):

```sh
vigil update --check    # compare installed vs latest release
vigil update            # verified download + checksum check + swap
```

Re-running the curl one-liner is always a safe in-place upgrade; your `~/.vigil` config, rules and extensions are never touched.

---

## Auto build and releases

| Event | What happens |
|---|---|
| push to `master` / `feat/**` | CI: fmt + clippy + tests on Linux, macOS, Windows |
| push to `master` | Docs site auto-deploys to GitHub Pages; snake animation refreshes |
| version tag `vX.Y.Z` | Full release: cross-compiled tarballs (5 targets) + SHA256SUMS |

Same version, new commits = CI green, nothing published. Bumped version = new tag, new release, new artifacts. Fully automatic.

---

## Architecture

```text
                ┌──────────────────────────────────────────┐
                │               vigil daemon               │
                │   ruleset · decision engine · event log  │
                └───────▲──────────────▲──────────────▲────┘
                        │              │              │
          ┌─────────────┘              │              └────────────┐
          │                            │                           │
 ┌────────┴───────┐          ┌─────────┴────────┐        ┌─────────┴────────┐
 │  agent hooks   │          │   audit watcher  │        │    vigil shim    │
 │ (native, block │          │ (fs watch, warn  │        │ (command wrapper,│
 │  before it runs)│         │  never blocks)   │        │  works with any  │
 └────────────────┘          └──────────────────┘        │  CLI agent)      │
                                                         └──────────────────┘
```

Hook verdicts are computed locally and synchronously - the daemon is a reporter, not a gatekeeper. Every matched event lands in the event log with the agent, action, rule id, and severity, visible in `vigil tui`.

---

## Extensions

Ship your own rules, patterns, and event hooks as a drop-in folder:

```toml
# ~/.vigil/extensions/my-team/manifest.toml
[extension]
name = "my-team"
version = "0.1.0"
description = "Company-internal rules"

# files alongside the manifest are picked up:
# rules/*.toml     - same schema as the core ruleset
# patterns/*.toml  - reusable masking patterns
# hooks/*.toml     - event hooks
```

Authoring guide: [Extensions](docs/extensions.md).

---

## Docs site

Full documentation lives in [docs/](docs/) and is published to the
[VigilFYR docs site](https://potenfyr-studios.github.io/VigilFYR/) with
installation, rules, masking, agents matrix, daemon, extensions and
security-model guides. Built with Next.js (`docs/site/`), statically
exported and auto-deployed on every push to `master`.

---

## ⭐ Star History

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=potenfyr-studios/vigilfyr&type=Date&theme=dark" />
  <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=potenfyr-studios/vigilfyr&type=Date" />
  <img alt="Star history chart for VigilFYR" src="https://api.star-history.com/svg?repos=potenfyr-studios/vigilfyr&type=Date" width="80%" />
</picture>

Live VigilFYR star chart, served by [star-history.com](https://star-history.com).

---

## Contributing

Contributions make the open-source community such an amazing place to learn, inspire and create. Any contributions you make are **greatly appreciated** - see [CONTRIBUTING.md](CONTRIBUTING.md) and the [good first issues](https://github.com/PotenFYR-Studios/VigilFYR/labels/good%20first%20issue). Security concerns: please use [SECURITY.md](SECURITY.md) (private vulnerability reporting), not public issues.

🐛 **Bug reports**: [open an issue](https://github.com/PotenFYR-Studios/VigilFYR/issues/new/choose).

<a href="https://github.com/PotenFYR-Studios/VigilFYR/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=PotenFYR-Studios/VigilFYR" alt="VigilFYR contributors" />
</a>
<a href="https://github.com/PotenFYR-Studios/VigilFYR/stargazers">
  <img src="https://img.shields.io/github/stars/PotenFYR-Studios/VigilFYR?style=social&label=Stars" alt="Live star count" />
</a>
<a href="https://github.com/PotenFYR-Studios/VigilFYR/network/members">
  <img src="https://img.shields.io/github/forks/PotenFYR-Studios/VigilFYR?style=social&label=Forks" alt="Live fork count" />
</a>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/output/github-snake-dark.svg" />
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/output/github-snake.svg" />
  <img alt="Contribution snake animation" src="https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/output/github-snake.svg" width="100%" />
</picture>

---

## Platform Support

Capability-based detection rather than distro assumptions. Works wherever Rust stable builds and the agent exists: Linux (systemd user units for autostart), macOS (launchd), Windows (autostart + tray). Release targets: x86_64 and arm64 across all three OSes.

Vigil is fully local: no telemetry, no cloud calls - the only outbound requests are the rule feed and update checks, both configurable off.

---

## License

Vigil is licensed under **Apache-2.0 with Commons Clause**.

- Free to use, study, modify, self-host and redistribute
- Commercial use is welcome: embedding Vigil as a feature inside a
  larger product or service is explicitly allowed
- What is NOT allowed: selling Vigil itself - offering the software,
  or a service whose value derives entirely or substantially from it,
  as a paid product (this includes managed-hosting-of-Vigil alone)

This matches the whole PotenFYR-Studios org: the tool stays open and
auditable, and nobody gets to resell it as-is. The [LICENSE](LICENSE) file
(https://github.com/PotenFYR-Studios/VigilFYR/blob/master/LICENSE) is the
authoritative legal text; this section is a plain-English summary.

### Restrictions at a glance

| | Allowed | Not allowed |
|---|:---:|:---:|
| Personal / internal use | ✅ | |
| Self-hosting for your company | ✅ | |
| Modifying and redistributing (same license) | ✅ | |
| Embedding Vigil as a feature of a larger paid product | ✅ | |
| Building paid services around Vigil | ✅ | |
| Selling Vigil itself (or a copy) for a fee | | ❌ |
| Offering paid managed hosting of Vigil alone | | ❌ |
| Paid support/consulting whose value is Vigil itself | | ❌ |
| Removing LICENSE / attribution notices | | ❌ |

"Sell" here follows the Commons Clause definition: charging for a
product or service whose value derives entirely or substantially from
Vigil itself. If Vigil is a minor feature of something bigger, you
are fine. Questions or a commercial exception: [contact the org](https://potenfyr.in/).

---

<div align="center">

## 🎯 The Vigil Promise

✨ **Hook-enforced** (blocked before it runs, verdict in milliseconds) · 👁️ **Proxy-proof** (guards tool actions, not network) · 🛡️ **Fail-open safety** (a broken guard never breaks your agent) · 🤝 **Community-driven** (extensions, shared rule feed, open source)

[![GitHub](https://img.shields.io/badge/GitHub-PotenFYR--Studios-181717?style=for-the-badge&logo=github&logoColor=white&labelColor=1c1e26)](https://github.com/PotenFYR-Studios)
[![Website](https://img.shields.io/badge/Website-potenfyr.in-8b5cf6?style=for-the-badge&logo=googlechrome&logoColor=white&labelColor=1c1e26)](https://potenfyr.in/)
[![Community](https://img.shields.io/badge/Community-Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white&labelColor=1c1e26)](https://discord.com/invite/zUaN2FPBec)
[![Docs](https://img.shields.io/badge/Docs-VigilFYR%20docs-0ea5e9?style=for-the-badge&logo=readme&logoColor=white&labelColor=1c1e26)](https://potenfyr-studios.github.io/VigilFYR/)

<!-- markdownlint-disable -->
<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:ec4899,50:8b5cf6,100:0ea5e9&height=120&section=footer&text=Made%20with%20%E2%9D%A4%EF%B8%8F%20by%20PotenFYR%20Studios&fontSize=22&fontColor=ffffff&animation=twinkling" width="100%" alt="footer"/>

</div>
<!-- markdownlint-enable -->

</div>
