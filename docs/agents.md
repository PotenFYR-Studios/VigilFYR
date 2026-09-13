# Agents Matrix

Vigil guards agents at the highest level each one supports, with two universal fallbacks. `vigil setup` auto-detects installed agents and picks the mode; you can change any of it later.

| Agent | Enforcement (native hooks, auto-installed) | Audit (fs watch) | Shim (command wrapper) |
| :--- | :---: | :---: | :---: |
| Claude Code | 🪝 yes | ✅ | ✅ |
| Codex | 🪝 yes | ✅ | ✅ |
| Gemini CLI | 🪝 yes | ✅ | ✅ |
| Cursor | 🪝 yes | ✅ | ✅ |
| OpenCode | 🪝 yes | ✅ | ✅ |
| Hermes | 🪝 yes | ✅ | ✅ |
| any other agent / plain shell | — | ✅ | ✅ |

## The three layers

### Enforcement — hooks

For hook-capable agents, Vigil registers hook handlers during `vigil setup`. Every tool call is evaluated **before** it executes: reads, writes, searches, command execution, network-touching commands. Verdicts come back as exit codes — `0` allow, `2` deny, `3` ask — with `warn` and `mask` verdicts allowing the call while recording it.

This is the only layer that *blocks*. It covers exactly the agents listed above with hook support; hooks cover hookable agents, and nothing else can block.

### Audit — filesystem watch

The daemon watches sensitive areas and records access by any process, warning in the TUI. Audit mode never blocks: every `deny` verdict is demoted to `warn`. Use it to:

- calibrate new rules without breaking your workflow,
- cover agents without hook support,
- catch processes no agent config could ever reach.

### Shim — command wrapper

```sh
vigil shim <command> [args…]
```

Wraps an arbitrary command with the same rule evaluation. Point an unsupported agent's shell at it, or use it manually for one-off guarded runs. Same verdicts, same event log, same rules — just no ability to block mid-flight for tools it doesn't control.

## Per-agent modes

Configured in `~/.vigil/config.toml` (`agents` table) or via the CLI:

```sh
vigil config get agents.codex
vigil config set agents.codex enforce
vigil config set agents.gemini-cli audit
vigil config set agents.cursor off
vigil reload
```

| Mode | Behavior |
| :--- | :--- |
| `enforce` | Hooks active; denies block. Default for detected agents. |
| `audit` | Everything is recorded and warned; nothing is blocked. |
| `off` | No hooks installed; the audit watcher still sees filesystem activity. |

## Proxy-proofing

Vigil evaluates **local tool actions** — the files your agent's tools touch, the commands it runs. It does not inspect or care about LLM network traffic. Which API, proxy, gateway, or router serves the model (9router, LiteLLM, a corporate endpoint) has zero effect on verdicts. Same machine, same rules, same behavior regardless of what the model talks to.

## Honest limits

See [Security model & limitations](security-model.md) for what each layer can and cannot catch — in particular, hooks cover hookable agents, and audit/shim provide visibility and wrapping, not interception.
