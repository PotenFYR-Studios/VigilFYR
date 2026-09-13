# Security Model & Limitations

What Vigil guarantees, what it can't, and why it fails the way it fails.

## The model

Vigil is a **policy gate between an AI agent's tools and your filesystem**. Every tool action an agent attempts - read, write, search, exec, net - is evaluated against a ruleset before (hooks) or around (shim) execution, or observed after the fact (audit).

Three layers, honest about coverage:

| Layer | Blocks? | Covers | Weakness |
| :--- | :---: | :--- | :--- |
| **Hooks** | ✅ | Hook-capable agents (Claude Code, Codex, Gemini CLI, Cursor, OpenCode, Hermes) | Only what the agent reports through its hook API |
| **Shim** | ✅ (wraps) | Any command, explicitly wrapped | Only commands routed through `vigil shim` |
| **Audit** | ❌ (warns) | Any process touching watched areas | Detection after the fact, not prevention |

**Hooks cover hookable agents; audit and shim are for the rest.** There is no layer that blocks an arbitrary, unhooked, unshimmed process from reading a file - that's what OS permissions are for. Vigil complements them; it doesn't replace them.

## What the core ruleset protects

Sensitive areas denied out of the box: `.env` files, secrets directories, SSH/GnuPG/AWS directories, private keys and certificates, credentials files (`.npmrc`, `.netrc`, `.pypirc`), crypto wallet directories, system account files, cloud metadata endpoints, and catastrophic exec patterns (`rm -rf /`, `sudo` system writes, home-directory recursive deletes). See the [Rules reference](rules.md#built-in-rules-at-a-glance) for the exact list and severities.

## Fail-open, deliberately

**On daemon error, ruleset load error, or hook malfunction, Vigil allows the action** (exit code `0`). Rationale:

- Vigil sits in the critical path of every agent tool call. A guard that can fail closed can take your whole workflow down - a corrupted rule file or a dead daemon would make every agent useless.
- The threat model is *an AI agent fumbling toward your secrets*, not a determined attacker. An attacker who can disable Vigil can also delete it; fail-closed gains nothing against that adversary.

The cost: transient daemon failures are silent allowances. Audit logging records what it can; the TUI surfaces daemon health. If your threat model weighs availability lower than exposure, keep `general.mode = "audit"` off and treat Vigil as one layer among OS permissions, disk encryption, and secret managers.

## Verdict and exit code contract

| Verdict | Meaning | Hook exit code |
| :--- | :--- | :---: |
| `allow` | Permitted (explicit or default) | `0` |
| `deny` | Blocked before execution | `2` |
| `ask` | Deferred to the agent/user for confirmation | `3` |
| `warn` | Permitted, recorded, flagged in the TUI | `0` |
| `mask` | Permitted with sensitive content redacted | `0` |

First matching rule wins; file order is priority; no match allows (rule `default`). In audit mode every `deny` demotes to `warn`.

## Known limitations

- **Rule matching is pattern-based.** Globs and regexes can be evaded by exotic path encodings or command obfuscation. The exec rules accept known false negatives (e.g. `rm --help /`) and some read-overblocking (`sudo cat /etc/passwd` matches the sudo-write rule) - documented trade-offs in [`rules/core.toml`](../rules/core.toml).
- **Masking is best-effort content filtering**, not a DLP product. Novel secret formats not covered by the six pattern families pass through; custom patterns and extensions narrow the gap.
- **`net` scope is command-line level** (e.g. metadata endpoint hosts, curl-pipe-shell), not a firewall. It does not intercept arbitrary network syscalls.
- **Agent coverage is only as good as hook support.** A future agent without hooks gets audit + shim, which record and wrap but cannot block mid-flight.
- **Local trust boundary.** Vigil runs as your user with your permissions. It does not defend against malware running as you, only against agent tool actions.

## Reporting vulnerabilities

Follow [SECURITY.md](../SECURITY.md) - private vulnerability reporting, not public issues.
