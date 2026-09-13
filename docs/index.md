# Vigil Docs

The AI-agent guard: blocks Claude Code, Codex, Gemini CLI, Cursor, OpenCode, and Hermes from reading, writing, searching, or executing in sensitive areas - secrets, `.env`, keys, `.ssh`, `.aws`, wallets, system dirs.

## Pages

| Page | Contents |
| :--- | :--- |
| [Getting Started](getting-started.md) | Install one-liner, setup wizard walkthrough, first rules check |
| [Rules Reference](rules.md) | Rule schema, precedence, exit codes, cookbook, built-in rules table |
| [Masking](masking.md) | Opt-in sensitive-data redaction with 4-char prefix preserved |
| [Agents Matrix](agents.md) | Hook / audit / shim coverage per agent, per-agent modes, proxy-proofing |
| [Extensions](extensions.md) | `manifest.toml` layout and authoring guide |
| [Daemon, IPC & Live Report](daemon.md) | `vigil daemon`, Unix-socket bus, `vigil tui`, update checks |
| [Configuration Reference](configuration.md) | Full `config.toml` schema, `vigil config get/set`, file locations |
| [Security Model & Limitations](security-model.md) | Threat model, fail-open rationale, honest limits |

Quick start:

```sh
curl -fsSL https://raw.githubusercontent.com/PotenFYR-Studios/VigilFYR/master/install.sh | sh
vigil setup
```

> This content is also served by the [docs site](web/) built with Vite, React, TypeScript, Bun and Magic UI.

Portable archives cover Linux (GNU/musl on x86_64, ARM64, ARMv7, RISC-V64), macOS (x86_64/ARM64), and Windows (x64/ARM64). Two installers are published on every release: the `curl`/PowerShell one-line installers and native installers (`.deb`, `.pkg`, and `.msi`).
