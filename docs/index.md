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

> This content is also served by the [docs site](site/) (Magic UI scaffold in `docs/site/`, deployed via GitHub Pages).
