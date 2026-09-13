# Getting Started

Install Vigil, run the setup wizard once, and every supported AI coding agent on the machine is guarded.

## Install

The install script detects your OS and architecture, downloads the matching release binary, and verifies its checksum.

```sh
curl -fsSL https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/master/install.sh | sh
```

Supported platforms: **Linux, macOS, Windows** on **x86_64 and arm64**.

Prefer building from source:

```sh
git clone https://github.com/PotenFYR-Studios/VigilFYR
cd VigilFYR
cargo install --path .
```

Rust 1.75+ required (see `rust-toolchain.toml`).

## The setup wizard

`vigil setup` is interactive and covers everything:

1. **OS check** - verifies your platform is supported.
2. **Agent auto-detect** - scans for Claude Code, Codex, Gemini CLI, Cursor, OpenCode, and Hermes installs.
3. **Per-agent mode** - choose `enforce`, `audit`, or `off` for each detected agent. Default for detected agents is `enforce`.
4. **Extra paths** - add project-specific or personal paths beyond the built-in sensitive areas.
5. **Masking opt-in** - sensitive-data masking is off by default; opt in here (or later via config).
6. **Autostart + tray** - start the daemon on login and show the tray icon (update checks live there too).

Zero-prompt variant, all defaults:

```sh
vigil setup --defaults
```

## What runs where

- `vigil` with no arguments (or `vigil tui`) - the live report: verdicts, agents, rules, event log, export.
- `vigil daemon` - the background guard: audit watcher plus rule sync.
- `vigil intercept` - the hook entry point agents call before each tool action.

## First rules check

```sh
vigil rules list     # active rules, in priority order
vigil rules path     # where rules are loaded from
vigil reload         # re-read config + rules without restarting
```

Next: [Rules reference](rules.md) · [Agents matrix](agents.md) · [Configuration reference](configuration.md)
