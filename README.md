![VigilFYR Banner](https://capsule-render.vercel.app/api?type=waving&color=0:0ea5e9,50:8b5cf6,100:ec4899&height=220&section=header&text=VigilFYR&fontSize=56&fontColor=ffffff&fontAlignY=34&desc=The%20AI-agent%20guard.%20Every%20read.%20Every%20write.%20Every%20command.&descSize=18&descAlignY=55&animation=twinkling)

[![Typing SVG](https://readme-typing-svg.demolab.com?font=Fira+Code&weight=600&size=20&pause=1200&color=8B5CF6&center=true&vCenter=true&width=800&lines=Block+AI+agents+from+your+secrets+%26+keys;Hooks+for+Claude+Code%2C+Codex%2C+Gemini+CLI+%26+more;Audit+watch+%2B+shim+for+everything+else;Proxy-proof%3A+guards+local+tool+actions;Live+TUI+report+of+every+blocked+event;By+PotenFYR+Studios+%C2%B7+support%40potenfyr.in)](https://github.com/PotenFYR-Studios/VigilFYR)

<div align="center">

[![CI](https://img.shields.io/github/actions/workflow/status/PotenFYR-Studios/VigilFYR/ci.yml?style=flat-square&logo=githubactions&label=CI&labelColor=1c1e26&color=2ea043)](https://github.com/PotenFYR-Studios/VigilFYR/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/PotenFYR-Studios/VigilFYR?style=flat-square&logo=github&label=Release&labelColor=1c1e26&color=2ea043)](https://github.com/PotenFYR-Studios/VigilFYR/releases/latest)
[![License](https://img.shields.io/badge/License-Apache--2.0%20%2B%20Commons%20Clause-8b5cf6.svg?style=flat-square&logo=apache&logoColor=white&labelColor=1c1e26)](LICENSE)
[![Platforms](https://img.shields.io/badge/Platforms-linux%20%7C%20macOS%20%7C%20Windows-0ea5e9?style=flat-square&logo=linux&logoColor=white&labelColor=1c1e26)](#-quick-start)
[![Architectures](https://img.shields.io/badge/Architectures-x86__64%20%7C%20arm64-0ea5e9?style=flat-square&labelColor=1c1e26)](#-quick-start)

</div>

**Vigil** is a local guard for AI coding agents: it sits between your agent and your filesystem and blocks reads, writes, searches, and commands that touch sensitive areas — `.env` files, secrets directories, SSH and AWS credentials, private keys, crypto wallets, system paths. Hooks are auto-installed for agents that support them; a filesystem-watch audit mode and the `vigil shim` wrapper cover everything else. Because it guards the tool actions on your machine, it is **proxy-proof**: it works identically behind any LLM API proxy or router (9router, LiteLLM, corporate gateways) — the model never sees what the tool never touched.

[Getting Started](docs/getting-started.md) · [Rules Reference](docs/rules.md) · [Security Model](docs/security-model.md)

---

## 📑 Contents

- [✨ Highlights](#-highlights)
- [🚀 Quick Start](#-quick-start)
- [🛡️ How It Works](#️-how-it-works)
- [🤖 Supported Agents](#-supported-agents)
- [⚙️ Configuration](#️-configuration)
- [🧩 Extensions](#-extensions)
- [📜 Live Report](#-live-report)
- [🤝 Contributing](#-contributing)
- [📜 License](#-license)
- [⭐ Star History](#-star-history)

---

## ✨ Highlights

- 🔒 **Sensitive-area blocking out of the box**: a built-in core ruleset denies access to `.env` files, secrets dirs, `.ssh/`, `.aws/`, `.gnupg/`, key files, credentials files, wallet directories, and system paths — plus exec guards for `rm -rf /`, `sudo` system writes, and cloud metadata endpoints.
- 🪝 **Hook enforcement, auto-installed**: `vigil setup` detects your installed agents and wires up their hook systems so every tool call is checked before it runs. Exit codes follow a strict contract: `0` allow, `2` deny, `3` ask.
- 👁️ **Audit + shim fallback**: agents without hooks get the filesystem-watch audit mode (records and warns, never blocks) and the `vigil shim` command wrapper. Enforcement where possible, visibility everywhere.
- 🛡️ **Proxy-proof by design**: Vigil inspects local tool actions, not network traffic — so which LLM API, proxy, or router you use is irrelevant. Same rules, same verdicts, everywhere.
- 📜 **Live TUI report**: the default command, `vigil tui`, shows what was blocked, from whom, which agent, and why — every verdict with its rule id and severity. Event log included, export included.
- 🎭 **Optional masking** (off by default, you opt in): AWS keys, GitHub PATs, OpenAI keys, JWTs, private keys, and env values are redacted with a 4-character prefix preserved so you can still tell which secret it was.
- 🧩 **Extensions**: drop a `manifest.toml` into `~/.vigil/extensions/<name>/` to add your own rules, patterns, and event hooks.
- 🔄 **Remote rule sync**: rules sync from the community feed on boot and `vigil reload`; override any built-in rule by id from `~/.vigil/rules/` or `./.vigil/rules/`.

## 🚀 Quick Start

Install (OS and arch detected, checksum verified):

```sh
curl -fsSL https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/master/install.sh | sh
```

Sixty-second setup:

```sh
# Interactive wizard: OS check, agent auto-detect, per-agent mode,
# extra protected paths, masking opt-in, autostart + tray
vigil setup

# Or zero prompts, all defaults
vigil setup --defaults

# Check what your rules actually are
vigil rules list
vigil rules path
```

That's it. Agents you had installed are now guarded; `vigil tui` shows the live report. Supported: **Linux, macOS, Windows** on **x86_64 and arm64**.

## 🛡️ How It Works

```
                    ┌──────────────────────────────────────────┐
                    │                vigil daemon              │
                    │   ruleset · decision engine · event log  │
                    └───────▲──────────────▲──────────────▲────┘
                            │              │              │
              ┌─────────────┘              │              └────────────┐
              │                            │                           │
     ┌────────┴───────┐          ┌─────────┴────────┐        ┌─────────┴────────┐
     │  agent hooks   │          │   audit watcher  │        │    vigil shim    │
     │ (native, block │          │ (fs watch, warn  │        │ (command wrapper,│
     │  before it runs)│          │  never blocks)   │        │  works with any  │
     └────────────────┘          └──────────────────┘        │  CLI agent)      │                                                             └──────────────────┘
```

1. **Hooks** — for hook-capable agents, Vigil installs hook handlers automatically. Every tool call (read, write, search, exec, net) is evaluated against the ruleset *before* it executes. Verdicts: `deny`, `allow`, `warn`, `mask`. Exit codes: `0` allow, `2` deny, `3` ask. On daemon or ruleset error the hook **fails open** — a broken guard must never break your agent.
2. **Audit** — a filesystem watcher records sensitive-area access by any process and warns. Audit mode demotes every `deny` to `warn`, so nothing is ever blocked while you calibrate rules.
3. **Shim** — `vigil shim <command>…` wraps arbitrary commands with the same rule evaluation, for agents and tools with no hook system.

Every matched event lands in the event log with the agent, action, rule id, and severity — visible in `vigil tui`.

## 🤖 Supported Agents

| Agent | Enforcement | Audit | Shim |
| :--- | :---: | :---: | :---: |
| Claude Code | 🪝 native hooks | ✅ | ✅ |
| Codex | 🪝 native hooks | ✅ | ✅ |
| Gemini CLI | 🪝 native hooks | ✅ | ✅ |
| Cursor | 🪝 native hooks | ✅ | ✅ |
| OpenCode | 🪝 native hooks | ✅ | ✅ |
| Hermes | 🪝 native hooks | ✅ | ✅ |
| anything else | — | ✅ | ✅ |

`vigil setup` auto-detects which of these are installed and configures the best mode for each. Per-agent mode is one of `off`, `audit`, `enforce` (see [agents matrix](docs/agents.md)). Audit and shim coverage is universal — no agent gets a free pass; only enforcement depth differs.

## ⚙️ Configuration

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
| `masking.patterns` | all six | `aws_key`, `github_pat`, `openai_key`, `jwt`, `private_key`, `env_values` |
| `rules.remote_update` | `true` | Sync community rules on boot / `vigil reload` |
| `agents.<id>` | `claude-code=enforce` | Per-agent mode: `off`, `audit`, `enforce` |

Rules layering: built-in core ruleset → `~/.vigil/rules/*.toml` → `./.vigil/rules/*.toml` (later files override earlier ones **by rule id**; within a file, order is priority — first match wins). `vigil rules update` refreshes the remote feed; `vigil reload` applies everything without a restart; `vigil update` self-updates the binary (semver check plus commits-behind, surfaced in the tray).

Full schema and cookbook: [Rules Reference](docs/rules.md) · [Configuration Reference](docs/configuration.md).

## 🧩 Extensions

Ship your own rules, patterns, and event hooks as a drop-in folder:

```toml
# ~/.vigil/extensions/my-team/manifest.toml
[extension]
name = "my-team"
version = "0.1.0"
description = "Company-internal rules"

# files alongside the manifest are picked up:
# rules/*.toml     — same schema as the core ruleset
# patterns/*.toml  — reusable masking patterns
# hooks/*.toml     — event hooks
```

Authoring guide: [Extensions](docs/extensions.md).

## 📜 Live Report

`vigil tui` is the default command: every verdict, who triggered it, which agent, and why — rule id and severity included — with a scrollable event log and export.

> 📸 *Screenshot and asciinema recording coming here — the TUI is under active development.*

## 🤝 Contributing

Found a bug, want a new rule, or want to improve the docs? We welcome issues and pull requests!

1. Fork the repo and branch from `master`.
2. For rule changes, edit [`rules/core.toml`](rules/core.toml) and keep the first-match-wins ordering (most specific first).
3. Check locally: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
4. Open a PR; CI runs fmt, clippy, and tests on Linux, macOS, and Windows.

🐛 **Bug reports**: [open an issue](https://github.com/PotenFYR-Studios/VigilFYR/issues/new/choose).
🔐 **Security vulnerabilities**: please follow [SECURITY.md](SECURITY.md) — do not open public issues for security reports. See also [CONTRIBUTING.md](CONTRIBUTING.md).

## 📜 License

Vigil is free for any purpose, commercial use included: run it, fork it, modify it, self-host it, and build products or services around it. You may not sell the software itself, offer a paid product or service whose value derives entirely or substantially from this software's functionality, or use PotenFYR names or trademarks. License notices you redistribute must carry the Commons Clause notice. The [LICENSE](LICENSE) is the single authoritative source, not this summary.

---

Built by **[PotenFYR Studios](https://github.com/PotenFYR-Studios)** · Part of the PotenFYR Studios open-source ecosystem.

---

## ⭐ Star History

[![Star History Chart](https://api.star-history.com/svg?repos=PotenFYR-Studios/VigilFYR&type=Date)](https://star-history.com/#PotenFYR-Studios/VigilFYR&Date)

---

![footer](https://capsule-render.vercel.app/api?type=waving&color=0:ec4899,50:8b5cf6,100:0ea5e9&height=120&section=footer&text=VigilFYR%20%C2%B7%20PotenFYR%20Studios&fontSize=22&fontColor=ffffff&animation=twinkling)
